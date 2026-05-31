//! Shared input + batch-control plumbing for every write subcommand.
//!
//! Every write verb (`add`, `set`, `create *`) flattens [`BatchOptions`] into
//! its args struct via `#[clap(flatten)]`, giving the CLI a consistent
//! `[FILE...] [--dry-run] [--skip N] [--skip-error] [--skip-duplicates]`
//! tail. The free functions here ([`collect_batch`], [`execute_each`],
//! [`emit_dry_run`]) implement the matching pipeline so each verb's
//! handler stays focused on its library call.

use std::io::{self, Read};
use std::path::{Path, PathBuf};

use clap::Args;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::output::{green, write_json_pretty_stdout, yellow, OutputFormat};
use crate::BoxedErr;

const STDIN_MARKER: &str = "-";

/// Shared positional + batch-control options for every write subcommand.
///
/// Either supply a single record via the verb's per-field flags (where
/// available), or pass one or more positional JSON files (`-` reads stdin).
/// Each file may contain one object or a JSON array of objects matching the
/// parameter struct for the verb. The two modes are mutually exclusive.
#[derive(Args, Clone, Debug)]
pub(crate) struct BatchOptions {
    /// JSON input files (each holds one object or an array). `-` reads stdin.
    #[clap(value_name = "FILE", num_args = 0..)]
    pub files: Vec<PathBuf>,

    /// Validate inputs and print the would-apply plan without mutating
    #[clap(long)]
    pub dry_run: bool,

    /// Drop the first N items of the input batch (resume after a partial run)
    #[clap(long, value_name = "N", default_value_t = 0)]
    pub skip: usize,

    /// Continue past per-item failures at execution time
    #[clap(long)]
    pub skip_error: bool,

    /// Filter out items already present in MoneyMoney (add transaction only)
    #[clap(long)]
    pub skip_duplicates: bool,
}

/// Reject `--skip-duplicates` on subcommands where the dedup semantics don't apply.
pub(crate) fn reject_skip_duplicates(batch: &BatchOptions, verb: &str) -> Result<(), BoxedErr> {
    if batch.skip_duplicates {
        return Err(format!(
            "--skip-duplicates is only supported on `add transaction`, not on `{verb}`"
        )
        .into());
    }
    Ok(())
}

/// Collect a batch of records: either one item from flag-built params, or N items
/// parsed from the positional `[FILE...]` inputs. The two are mutually exclusive;
/// a bare invocation (neither flags nor files) is rejected with a friendly hint.
///
/// `--skip N` drops the first N items of the resulting batch.
pub(crate) fn collect_batch<P>(
    flag_built: Option<P>,
    batch: &BatchOptions,
) -> Result<Vec<P>, BoxedErr>
where
    P: DeserializeOwned,
{
    let has_files = !batch.files.is_empty();
    match (flag_built, has_files) {
        (Some(_), true) => {
            Err("field flags are mutually exclusive with positional JSON files".into())
        }
        (None, false) => {
            Err("provide JSON via stdin/file or use the field flags; see --help".into())
        }
        (Some(p), false) => {
            let mut items = vec![p];
            apply_skip(&mut items, batch.skip);
            Ok(items)
        }
        (None, true) => {
            let mut items: Vec<P> = Vec::new();
            for path in &batch.files {
                let raw = read_input(path)?;
                let value: serde_json::Value = serde_json::from_str(raw.trim())
                    .map_err(|e| format!("parsing JSON from {}: {e}", display_path(path)))?;
                match value {
                    serde_json::Value::Array(arr) => {
                        for (idx, v) in arr.into_iter().enumerate() {
                            let p: P = serde_json::from_value(v)
                                .map_err(|e| format!("{}[{idx}]: {e}", display_path(path)))?;
                            items.push(p);
                        }
                    }
                    other => {
                        let p: P = serde_json::from_value(other)
                            .map_err(|e| format!("{}: {e}", display_path(path)))?;
                        items.push(p);
                    }
                }
            }
            apply_skip(&mut items, batch.skip);
            Ok(items)
        }
    }
}

fn read_input(path: &Path) -> io::Result<String> {
    if path.as_os_str() == STDIN_MARKER {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        if buf.trim().is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "stdin was empty"));
        }
        Ok(buf)
    } else {
        std::fs::read_to_string(path)
            .map_err(|e| io::Error::other(format!("reading {}: {e}", path.display())))
    }
}

fn display_path(path: &Path) -> String {
    if path.as_os_str() == STDIN_MARKER {
        "<stdin>".to_string()
    } else {
        path.display().to_string()
    }
}

fn apply_skip<P>(items: &mut Vec<P>, skip: usize) {
    if skip > 0 {
        let drop = skip.min(items.len());
        items.drain(..drop);
    }
}

/// JSON envelope for `--dry-run` previews. All fields borrow so we don't clone.
#[derive(Serialize)]
struct DryRunReport<'a, T: Serialize> {
    would_apply: &'a [T],
    skipped_as_duplicate: &'a [T],
    skipped_by_flag: usize,
}

pub(crate) fn emit_dry_run<T: Serialize>(
    would_apply: &[T],
    duplicates: &[T],
    skipped_by_flag: usize,
    format: OutputFormat,
) -> Result<(), BoxedErr> {
    let report = DryRunReport {
        would_apply,
        skipped_as_duplicate: duplicates,
        skipped_by_flag,
    };
    match format {
        OutputFormat::Json => write_json_pretty_stdout(&report),
    }
}

/// Bundled context for [`execute_each`]: everything the summary line
/// needs that isn't the items themselves.
pub(crate) struct ExecuteContext<'a> {
    pub skip_error: bool,
    pub color: bool,
    pub format: OutputFormat,
    pub duplicates_dropped: usize,
    pub skipped_by_flag: usize,
    pub verb: &'a str,
}

/// Apply `f` to each item, honoring `--skip-error`, then emit either a
/// JSON summary (when output format is JSON and the batch had multiple
/// items) or a colored one-line stderr summary.
pub(crate) fn execute_each<P, F>(
    items: Vec<P>,
    ctx: ExecuteContext<'_>,
    mut apply: F,
) -> Result<(), BoxedErr>
where
    F: FnMut(P) -> Result<(), moneymoney::Error>,
{
    let ExecuteContext {
        skip_error,
        color,
        format,
        duplicates_dropped,
        skipped_by_flag,
        verb,
    } = ctx;
    let total = items.len();
    let mut applied = 0usize;
    let mut errors: Vec<String> = Vec::new();
    for (idx, p) in items.into_iter().enumerate() {
        match apply(p) {
            Ok(()) => applied += 1,
            Err(e) => {
                if skip_error {
                    errors.push(format!("item {idx}: {e}"));
                } else {
                    return Err(format!("item {idx}: {e}").into());
                }
            }
        }
    }

    let multi = total + duplicates_dropped + skipped_by_flag > 1;
    let json_output = matches!(format, OutputFormat::Json);

    if json_output && multi {
        let summary = serde_json::json!({
            "verb": verb,
            "applied": applied,
            "duplicatesDropped": duplicates_dropped,
            "skippedByFlag": skipped_by_flag,
            "errors": errors,
        });
        write_json_pretty_stdout(&summary)?;
    } else if multi {
        let label = green("applied", color);
        let dup_part = if duplicates_dropped > 0 {
            format!("; {duplicates_dropped} skipped as duplicate")
        } else {
            String::new()
        };
        let err_part = if !errors.is_empty() {
            format!("; {} failed", errors.len())
        } else {
            String::new()
        };
        eprintln!("{label}: {applied} {verb} item(s){dup_part}{err_part}");
    }

    if !errors.is_empty() {
        let warn = yellow("warning", color);
        for err in &errors {
            eprintln!("{warn}: {err}");
        }
    }

    Ok(())
}

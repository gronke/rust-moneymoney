//! `create` subcommand tree (experimental): SEPA bank transfer and direct
//! debit. Both verbs accept JSON input only — their SEPA parameter shapes
//! have too many fields for ergonomic flag-mode.

use clap::{Args, Subcommand};
use moneymoney::create_bank_transfer::CreateBankTransferParams;
use moneymoney::create_direct_debit::CreateDirectDebitParams;
use serde::Serialize;

use crate::batch::{collect_batch, emit_dry_run, reject_skip_duplicates, BatchOptions};
use crate::output::{write_json_pretty_stdout, yellow, OutputFormat};
use crate::BoxedErr;

#[derive(Subcommand)]
pub(crate) enum CreateTarget {
    /// Create a SEPA bank transfer from JSON
    #[clap(name = "bank-transfer")]
    BankTransfer(BankTransferArgs),
    /// Create a SEPA direct debit from JSON
    #[clap(name = "direct-debit")]
    DirectDebit(DirectDebitArgs),
}

#[derive(Args)]
#[clap(
    about = "Create a SEPA bank transfer from JSON",
    long_about = "Create one or more SEPA bank transfers. Each JSON input is either a single \
object or an array of `CreateBankTransferParams`. Use `-` to read stdin."
)]
pub(crate) struct BankTransferArgs {
    #[clap(flatten)]
    pub batch: BatchOptions,
}

#[derive(Args)]
#[clap(
    about = "Create a SEPA direct debit from JSON",
    long_about = "Create one or more SEPA direct debits. Each JSON input is either a single \
object or an array of `CreateDirectDebitParams`. Use `-` to read stdin."
)]
pub(crate) struct DirectDebitArgs {
    #[clap(flatten)]
    pub batch: BatchOptions,
}

pub(crate) fn run(target: CreateTarget, format: OutputFormat, color: bool) -> Result<(), BoxedErr> {
    match target {
        CreateTarget::BankTransfer(args) => {
            reject_skip_duplicates(&args.batch, "create bank-transfer")?;
            if args.batch.files.is_empty() {
                crate::print_help_and_exit(&["create", "bank-transfer"]);
            }
            let items: Vec<CreateBankTransferParams> = collect_batch(None, &args.batch)?;
            run_create(items, &args.batch, format, color, moneymoney::create_bank_transfer)
        }
        CreateTarget::DirectDebit(args) => {
            reject_skip_duplicates(&args.batch, "create direct-debit")?;
            if args.batch.files.is_empty() {
                crate::print_help_and_exit(&["create", "direct-debit"]);
            }
            let items: Vec<CreateDirectDebitParams> = collect_batch(None, &args.batch)?;
            run_create(items, &args.batch, format, color, moneymoney::create_direct_debit)
        }
    }
}

fn run_create<P, F, R>(
    items: Vec<P>,
    batch: &BatchOptions,
    format: OutputFormat,
    color: bool,
    mut apply: F,
) -> Result<(), BoxedErr>
where
    P: Serialize,
    F: FnMut(P) -> Result<R, moneymoney::Error>,
    R: Serialize,
{
    if batch.dry_run {
        return emit_dry_run(&items, &[], batch.skip, format);
    }

    let total = items.len();
    let mut results: Vec<R> = Vec::with_capacity(total);
    let mut errors: Vec<String> = Vec::new();
    for (idx, p) in items.into_iter().enumerate() {
        match apply(p) {
            Ok(r) => results.push(r),
            Err(e) => {
                if batch.skip_error {
                    errors.push(format!("item {idx}: {e}"));
                } else {
                    return Err(format!("item {idx}: {e}").into());
                }
            }
        }
    }

    match format {
        OutputFormat::Json => write_json_pretty_stdout(&results)?,
    }
    if !errors.is_empty() {
        let warn = yellow("warning", color);
        for err in &errors {
            eprintln!("{warn}: {err}");
        }
        eprintln!(
            "{warn}: {} of {total} items failed (continued past errors via --skip-error)",
            errors.len()
        );
    }
    Ok(())
}

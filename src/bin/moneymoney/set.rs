//! `set transaction` subcommand: edit checkmark / category / comment on
//! existing transactions, one at a time or in batches.

use clap::{Args, Subcommand, ValueEnum};
use moneymoney::set_transaction::SetTransactionParams;

use crate::batch::{
    collect_batch, emit_dry_run, execute_each, reject_skip_duplicates, BatchOptions, ExecuteContext,
};
use crate::output::OutputFormat;
use crate::BoxedErr;

#[derive(Subcommand)]
pub(crate) enum SetTarget {
    /// Modify checkmark, category, or comment of an existing transaction
    Transaction(SetTransactionArgs),
}

#[derive(Args)]
#[clap(
    about = "Modify checkmark, category, or comment of an existing transaction",
    long_about = "Modify properties of an existing transaction by its `id` (as returned by \
`export transactions`).

For a single tweak, pass `--id` plus at least one of `--checkmark`, `--category`, `--comment`. \
For batch edits, pass one or more JSON files positionally (`-` reads stdin); each input may \
be a single object or a JSON array. The two modes are mutually exclusive.",
    after_help = "EXAMPLES:
    moneymoney set transaction --id 421337 --checkmark on
    moneymoney set transaction --id 421337 --category \"Food\\\\Coffee\" --comment reviewed

    moneymoney set transaction edits.json
    echo '[{\"id\":1,\"checkmarkTo\":\"on\"}]' | moneymoney set transaction -"
)]
pub(crate) struct SetTransactionArgs {
    /// Transaction ID (from `export transactions`)
    #[clap(long = "id", value_name = "ID")]
    pub id: Option<u64>,
    /// Set checkmark state to `on` or `off`
    #[clap(long = "checkmark", value_enum, value_name = "STATE")]
    pub checkmark: Option<CheckmarkState>,
    /// Assign or rename the category (nested with backslashes, e.g. `Food\\Coffee`)
    #[clap(long = "category", value_name = "NAME")]
    pub category: Option<String>,
    /// Replace the comment text (use `""` to clear)
    #[clap(long = "comment", value_name = "TEXT")]
    pub comment: Option<String>,
    #[clap(flatten)]
    pub batch: BatchOptions,
}

/// Checkmark state for `set transaction --checkmark`.
#[derive(ValueEnum, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CheckmarkState {
    On,
    Off,
}

impl CheckmarkState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

pub(crate) fn run(
    args: SetTransactionArgs,
    format: OutputFormat,
    color: bool,
) -> Result<(), BoxedErr> {
    reject_skip_duplicates(&args.batch, "set transaction")?;
    let flag_built = build_from_flags(&args)?;
    if flag_built.is_none() && args.batch.files.is_empty() {
        crate::print_help_and_exit(&["set", "transaction"]);
    }
    let items = collect_batch(flag_built, &args.batch)?;

    if args.batch.dry_run {
        return emit_dry_run::<SetTransactionParams>(&items, &[], args.batch.skip, format);
    }

    execute_each(
        items,
        ExecuteContext {
            skip_error: args.batch.skip_error,
            color,
            format,
            duplicates_dropped: 0,
            skipped_by_flag: args.batch.skip,
            verb: "set transaction",
        },
        moneymoney::set_transaction,
    )
}

pub(crate) fn build_from_flags(
    args: &SetTransactionArgs,
) -> Result<Option<SetTransactionParams>, BoxedErr> {
    let any_field_set = args.id.is_some()
        || args.checkmark.is_some()
        || args.category.is_some()
        || args.comment.is_some();
    if !any_field_set {
        return Ok(None);
    }
    let id = args.id.ok_or("--id is required (with field-flag mode)")?;
    if args.checkmark.is_none() && args.category.is_none() && args.comment.is_none() {
        return Err("at least one of --checkmark, --category, or --comment must be supplied".into());
    }
    let mut params = SetTransactionParams::new(id);
    if let Some(state) = args.checkmark {
        params = params.checkmark(state.as_str());
    }
    if let Some(category) = args.category.clone() {
        params = params.category(category);
    }
    if let Some(comment) = args.comment.clone() {
        params = params.comment(comment);
    }
    Ok(Some(params))
}

//! `add transaction` subcommand: insert one or many transactions into an
//! offline account.

use std::collections::BTreeSet;

use chrono::{NaiveDate, Utc};
use clap::{Args, Subcommand};
use moneymoney::add_transaction::AddTransactionParams;
use moneymoney::export_transactions::{ExportTransactionsParams, MoneymoneyTransaction};

use crate::batch::{collect_batch, emit_dry_run, execute_each, BatchOptions, ExecuteContext};
use crate::output::OutputFormat;
use crate::BoxedErr;

#[derive(Subcommand)]
pub(crate) enum AddTarget {
    /// Add a transaction to an offline account
    Transaction(AddTransactionArgs),
}

#[derive(Args)]
#[clap(
    about = "Add a transaction to an offline account",
    long_about = "Add a transaction to an offline account.

The four required fields (`--account`, `--date`, `--name`, `--amount`) can be supplied as flags \
for a single-item insert, or one or more JSON files (`-` reads stdin) can be passed positionally \
for batch insert. The two modes are mutually exclusive.

Each JSON input may be a single object or a JSON array of objects. Files are validated as a \
whole before any insert; a parse failure aborts the entire batch.",
    after_help = "EXAMPLES:
    # expense (negative amount)
    moneymoney add transaction --account test-cash --date 2026-05-17 \\
        --name \"Coffee Shop\" --amount -3.50 --purpose \"Latte\"

    # income (positive amount)
    moneymoney add transaction --account test-cash --date 2026-05-17 \\
        --name \"Employer GmbH\" --amount 2000.00

    moneymoney add transaction batch.json
    moneymoney add transaction a.json b.json
    cat batch.json | moneymoney add transaction -

    moneymoney add transaction --dry-run --skip-duplicates batch.json
    moneymoney add transaction --skip 5 --skip-error batch.json"
)]
pub(crate) struct AddTransactionArgs {
    /// Target account (UUID, IBAN, account name, or account number)
    #[clap(long = "account", value_name = "UUID|IBAN|NAME")]
    pub account: Option<String>,
    /// Booking date (YYYY-MM-DD)
    #[clap(long = "date", value_name = "YYYY-MM-DD")]
    pub date: Option<NaiveDate>,
    /// Counterparty name (matches the `name` field returned by `export transactions`)
    #[clap(
        long = "name",
        value_name = "NAME",
        conflicts_with_all = ["to", "from"],
        long_help = "Name of the transaction counterparty (payee for expenses, payer for income). \
                     Matches the `name` field returned by `moneymoney export transactions`. \
                     Hidden aliases `--to` and `--from` are accepted for users coming from the \
                     MoneyMoney AppleScript docs; mixing `--name` with either alias is rejected."
    )]
    pub name: Option<String>,
    /// Hidden alias for `--name` (sdef-style naming for expenses).
    #[clap(
        long = "to",
        value_name = "NAME",
        hide = true,
        conflicts_with_all = ["name", "from"]
    )]
    pub to: Option<String>,
    /// Hidden alias for `--name` (sdef-style naming for income).
    #[clap(
        long = "from",
        value_name = "NAME",
        hide = true,
        conflicts_with_all = ["name", "to"]
    )]
    pub from: Option<String>,
    /// Amount (negative = expense, positive = income)
    #[clap(long = "amount", value_name = "DECIMAL", allow_hyphen_values = true)]
    pub amount: Option<f64>,
    /// Purpose / description text (optional)
    #[clap(long = "purpose", value_name = "TEXT")]
    pub purpose: Option<String>,
    /// Category name (optional; nested with backslashes, e.g. `Food\\Coffee`)
    #[clap(long = "category", value_name = "NAME")]
    pub category: Option<String>,
    #[clap(flatten)]
    pub batch: BatchOptions,
}

pub(crate) fn run(
    args: AddTransactionArgs,
    format: OutputFormat,
    color: bool,
) -> Result<(), BoxedErr> {
    let flag_built = build_from_flags(&args)?;
    if flag_built.is_none() && args.batch.files.is_empty() {
        crate::print_help_and_exit(&["add", "transaction"]);
    }
    let items = collect_batch(flag_built, &args.batch)?;

    let (items, duplicates) = if args.batch.skip_duplicates {
        dedup_against_existing(items)?
    } else {
        (items, Vec::new())
    };

    if args.batch.dry_run {
        return emit_dry_run(&items, &duplicates, args.batch.skip, format);
    }

    execute_each(
        items,
        ExecuteContext {
            skip_error: args.batch.skip_error,
            color,
            format,
            duplicates_dropped: duplicates.len(),
            skipped_by_flag: args.batch.skip,
            verb: "add transaction",
        },
        moneymoney::add_transaction,
    )
}

pub(crate) fn build_from_flags(
    args: &AddTransactionArgs,
) -> Result<Option<AddTransactionParams>, BoxedErr> {
    let any_field_set = args.account.is_some()
        || args.date.is_some()
        || args.name.is_some()
        || args.to.is_some()
        || args.from.is_some()
        || args.amount.is_some()
        || args.purpose.is_some()
        || args.category.is_some();
    if !any_field_set {
        return Ok(None);
    }
    let account = args
        .account
        .clone()
        .ok_or("--account is required (with field-flag mode)")?;
    let date = args
        .date
        .ok_or("--date is required (with field-flag mode)")?;
    let name = args
        .name
        .clone()
        .or_else(|| args.to.clone())
        .or_else(|| args.from.clone())
        .ok_or("--name is required (with field-flag mode)")?;
    let amount = args
        .amount
        .ok_or("--amount is required (with field-flag mode)")?;
    let mut params = AddTransactionParams::new(account, date, name, amount);
    if let Some(purpose) = args.purpose.clone() {
        params = params.purpose(purpose);
    }
    if let Some(category) = args.category.clone() {
        params = params.category(category);
    }
    Ok(Some(params))
}

/// Drop batch items already present in MoneyMoney within the date range
/// spanned by the batch. All items must target a single account so the
/// export query can be scoped to it.
pub(crate) fn dedup_against_existing(
    items: Vec<AddTransactionParams>,
) -> Result<(Vec<AddTransactionParams>, Vec<AddTransactionParams>), BoxedErr> {
    if items.is_empty() {
        return Ok((items, Vec::new()));
    }
    let first_account = items[0].to_account.clone();
    if items.iter().any(|p| p.to_account != first_account) {
        return Err("--skip-duplicates requires all batch items to share the same --account \
                    (target account); split the batch per account and run again"
            .into());
    }
    let min_date = items.iter().map(|p| p.on_date).min().unwrap();
    let max_date = items.iter().map(|p| p.on_date).max().unwrap();

    let mut export_params = ExportTransactionsParams::new(min_date);
    export_params.to_date = Some(max_date);
    export_params.from_account = Some(first_account);
    let response = moneymoney::export_transactions(export_params)?;

    let existing: BTreeSet<(NaiveDate, String, i64)> = response
        .transactions
        .iter()
        .map(dedup_key_existing)
        .collect();

    let (kept, duplicates) = items.into_iter().partition::<Vec<_>, _>(|p| {
        !existing.contains(&(p.on_date, p.to.clone(), amount_key(p.amount)))
    });
    Ok((kept, duplicates))
}

fn dedup_key_existing(t: &MoneymoneyTransaction) -> (NaiveDate, String, i64) {
    let date = t.booking_date.with_timezone(&Utc).date_naive();
    (date, t.name.clone(), amount_key(t.amount))
}

/// Bucket `f64` amounts into hundredths-of-the-unit integer keys to compare safely.
fn amount_key(a: f64) -> i64 {
    (a * 100.0).round() as i64
}

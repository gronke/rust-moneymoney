//! `export` subcommand tree: accounts / categories / transactions / portfolio.

use chrono::NaiveDate;
use clap::{Args, Subcommand};
use moneymoney::export_portfolio::ExportPortfolioParams;
use moneymoney::export_transactions::ExportTransactionsParams;
use serde::Serialize;

use crate::filters::{AccountFilter, IconGroupOptions};
use crate::output::{write_json_pretty_stdout, OutputFormat};
use crate::BoxedErr;

#[derive(Subcommand)]
pub(crate) enum ExportTarget {
    /// Export accounts (balances, metadata) to stdout
    Accounts(ExportAccountsArgs),
    /// Export categories (budgets, metadata) to stdout
    Categories(ExportCategoriesArgs),
    /// Export transactions for a date range to stdout
    Transactions(ExportTransactionsArgs),
    /// Export portfolio securities (holdings, market values) to stdout
    Portfolio(ExportPortfolioArgs),
}

#[derive(Args)]
#[clap(
    about = "Export account balances and metadata to stdout",
    after_help = "EXAMPLES:
    moneymoney export accounts
    moneymoney export accounts --include-groups
    moneymoney export accounts --include-icon-data"
)]
pub(crate) struct ExportAccountsArgs {
    #[clap(flatten)]
    pub options: IconGroupOptions,
}

#[derive(Args)]
#[clap(
    about = "Export categories and budgets to stdout",
    after_help = "EXAMPLES:
    moneymoney export categories
    moneymoney export categories --include-groups
    moneymoney export categories --include-icon-data"
)]
pub(crate) struct ExportCategoriesArgs {
    #[clap(flatten)]
    pub options: IconGroupOptions,
}

#[derive(Args)]
#[clap(
    about = "Export transactions for a date range to stdout",
    after_help = "EXAMPLES:
    moneymoney export transactions --from-date 2024-01-01
    moneymoney export transactions --from-date 2024-01-01 --to-date 2024-12-31
    moneymoney export transactions --from-date 2024-06-01 --from-account <uuid-or-iban>"
)]
pub(crate) struct ExportTransactionsArgs {
    /// Inclusive start of the date range (YYYY-MM-DD)
    #[clap(long = "from-date", value_name = "YYYY-MM-DD")]
    pub from_date: NaiveDate,
    /// Inclusive end of the date range (YYYY-MM-DD)
    #[clap(long = "to-date", value_name = "YYYY-MM-DD")]
    pub to_date: Option<NaiveDate>,
    #[clap(flatten)]
    pub account: AccountFilter,
    /// Restrict to one category name
    #[clap(long = "from-category", value_name = "NAME")]
    pub from_category: Option<String>,
}

#[derive(Args)]
#[clap(
    about = "Export portfolio securities (holdings, market values) to stdout",
    after_help = "EXAMPLES:
    moneymoney export portfolio
    moneymoney export portfolio --from-account <uuid-or-iban>
    moneymoney export portfolio --from-asset-class Aktien"
)]
pub(crate) struct ExportPortfolioArgs {
    #[clap(flatten)]
    pub account: AccountFilter,
    /// Restrict to one asset class
    #[clap(long = "from-asset-class", value_name = "NAME")]
    pub from_asset_class: Option<String>,
}

pub(crate) fn run(target: ExportTarget, format: OutputFormat) -> Result<(), BoxedErr> {
    match target {
        ExportTarget::Accounts(args) => {
            let accounts = moneymoney::export_accounts()?;
            let accounts = if args.options.include_groups {
                accounts
            } else {
                accounts.into_iter().filter(|a| !a.group).collect()
            };
            emit_export(&accounts, args.options.include_icon_data, format)
        }
        ExportTarget::Categories(args) => {
            let categories = moneymoney::export_categories()?;
            let categories = if args.options.include_groups {
                categories
            } else {
                categories.into_iter().filter(|c| !c.group).collect()
            };
            emit_export(&categories, args.options.include_icon_data, format)
        }
        ExportTarget::Transactions(args) => {
            let mut params = ExportTransactionsParams::new(args.from_date);
            params.to_date = args.to_date;
            params.from_account = args.account.from_account;
            params.from_category = args.from_category;
            let response = moneymoney::export_transactions(params)?;
            match format {
                OutputFormat::Json => write_json_pretty_stdout(&response),
            }
        }
        ExportTarget::Portfolio(args) => {
            let mut params = ExportPortfolioParams::new();
            params.from_account = args.account.from_account;
            params.from_asset_class = args.from_asset_class;
            let response = moneymoney::export_portfolio(params)?;
            match format {
                OutputFormat::Json => write_json_pretty_stdout(&response),
            }
        }
    }
}

fn emit_export<T: Serialize>(
    items: &[T],
    include_icon_data: bool,
    format: OutputFormat,
) -> Result<(), BoxedErr> {
    match format {
        OutputFormat::Json => {
            let json = export_json_value_without_icons(items, include_icon_data)?;
            write_json_pretty_stdout(&json)
        }
    }
}

/// Serialize export items for CLI JSON: drop `icon` unless requested.
pub(crate) fn export_json_value_without_icons<T: Serialize>(
    items: &[T],
    include_icon_data: bool,
) -> Result<serde_json::Value, serde_json::Error> {
    let mut v = serde_json::to_value(items)?;
    if !include_icon_data {
        if let Some(items) = v.as_array_mut() {
            for item in items {
                if let Some(obj) = item.as_object_mut() {
                    obj.remove("icon");
                }
            }
        }
    }
    Ok(v)
}

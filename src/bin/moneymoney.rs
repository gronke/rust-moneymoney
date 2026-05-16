//! Command-line interface to MoneyMoney (macOS).
//!
//! Build with `--features experimental` to enable `create bank-transfer`.

use std::io::{self, Write};
use std::str::FromStr;
#[cfg(feature = "experimental")]
use std::{io::Read, path::PathBuf};

use chrono::NaiveDate;
use clap::{ArgEnum, Args, Parser, Subcommand};
use moneymoney::export_transactions::ExportTransactionsParams;
use serde::Serialize;

/// Clap `long_help` for the `--format` flag on export subcommands.
const EXPORT_FORMAT_LONG_HELP: &str =
    "Serialization format written to stdout. `json` is the default; additional formats \
     (e.g. CSV) may be added in future releases.";

#[derive(Parser)]
#[clap(
    name = "moneymoney",
    version,
    about = "Talk to the MoneyMoney app from the terminal"
)]
struct Cli {
    #[clap(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Read data from MoneyMoney
    Export {
        #[clap(subcommand)]
        target: ExportTarget,
    },
    /// Create payments (requires building with `--features experimental`)
    #[cfg(feature = "experimental")]
    Create {
        #[clap(subcommand)]
        target: CreateTarget,
    },
}

#[derive(Subcommand)]
enum ExportTarget {
    /// Export accounts (balances, metadata) to stdout
    Accounts(ExportAccountsArgs),
    /// Export categories (budgets, metadata) to stdout
    Categories(ExportCategoriesArgs),
    /// Export transactions for a date range to stdout
    Transactions(ExportTransactionsArgs),
}

#[derive(Args)]
#[clap(
    about = "Export account balances and metadata to stdout",
    long_about = "Export account balances and metadata to stdout.

Output encoding is selected with `--format` (default: json). By default, account groups \
(organizational folders) and per-account icon bytes are omitted. Use the flags below to \
include them.",
    after_help = "EXAMPLES:
    moneymoney export accounts
    moneymoney export accounts --include-group-accounts
    moneymoney export accounts --include-icon-data"
)]
struct ExportAccountsArgs {
    /// Output serialization format (`json` by default)
    #[clap(
        long,
        arg_enum,
        default_value_t = OutputFormat::Json,
        long_help = EXPORT_FORMAT_LONG_HELP
    )]
    format: OutputFormat,
    /// Include per-account icon bytes (omitted by default)
    #[clap(
        long = "include-icon-data",
        help = "Include per-account icon bytes (omitted by default)",
        long_help = "Include the `icon` field (raw image bytes) for each account. Omitted by default because \
                     payloads are large."
    )]
    include_icon_data: bool,
    /// Include account groups (omitted by default)
    #[clap(
        long = "include-group-accounts",
        help = "Include account groups / folders (omitted by default)",
        long_help = "Include account groups (`group: true`) in the output. Omitted by default; only real \
                     accounts (giro, savings, credit card, etc.) are exported."
    )]
    include_group_accounts: bool,
}

#[derive(Args)]
#[clap(
    about = "Export categories and budgets to stdout",
    long_about = "Export categories and budgets to stdout.

Output encoding is selected with `--format` (default: json). By default, category groups \
(organizational folders) and per-category icon bytes are omitted. Use the flags below to \
include them.",
    after_help = "EXAMPLES:
    moneymoney export categories
    moneymoney export categories --include-group-categories
    moneymoney export categories --include-icon-data"
)]
struct ExportCategoriesArgs {
    /// Output serialization format (`json` by default)
    #[clap(
        long,
        arg_enum,
        default_value_t = OutputFormat::Json,
        long_help = EXPORT_FORMAT_LONG_HELP
    )]
    format: OutputFormat,
    /// Include per-category icon bytes (omitted by default)
    #[clap(
        long = "include-icon-data",
        help = "Include per-category icon bytes (omitted by default)",
        long_help = "Include the `icon` field (raw image bytes) for each category. Omitted by default because \
                     payloads are large."
    )]
    include_icon_data: bool,
    /// Include category groups (omitted by default)
    #[clap(
        long = "include-group-categories",
        help = "Include category groups / folders (omitted by default)",
        long_help = "Include category groups (`group: true`) in the output. Omitted by default; only real \
                     categories are exported."
    )]
    include_group_categories: bool,
}

#[derive(Args)]
#[clap(
    about = "Export transactions for a date range to stdout",
    long_about = "Export transactions for a date range to stdout.

Output encoding is selected with `--format` (default: json). `--from-date` is required. \
Other filters are optional; when omitted, MoneyMoney applies its own defaults (e.g. no \
end date limit, all accounts, all categories).",
    after_help = "EXAMPLES:
    moneymoney export transactions --from-date 2024-01-01
    moneymoney export transactions --from-date 2024-01-01 --to-date 2024-12-31
    moneymoney export transactions --from-date 2024-06-01 --from-account <uuid-or-iban>"
)]
struct ExportTransactionsArgs {
    /// Inclusive start of the date range (YYYY-MM-DD)
    #[clap(
        long = "from-date",
        value_name = "YYYY-MM-DD",
        long_help = "Inclusive start date of the export range, in ISO 8601 calendar form (YYYY-MM-DD)."
    )]
    from_date: String,
    /// Inclusive end of the date range (YYYY-MM-DD)
    #[clap(
        long = "to-date",
        value_name = "YYYY-MM-DD",
        long_help = "Inclusive end date of the export range (YYYY-MM-DD). When omitted, MoneyMoney does not \
                     set an upper date bound."
    )]
    to_date: Option<String>,
    /// Restrict to one account (UUID or IBAN)
    #[clap(
        long = "from-account",
        value_name = "UUID|IBAN",
        long_help = "Only return transactions for this account. Accepts a MoneyMoney account UUID or IBAN. \
                     When omitted, transactions from all accounts are included."
    )]
    from_account: Option<String>,
    /// Restrict to one category name
    #[clap(
        long = "from-category",
        value_name = "NAME",
        long_help = "Only return transactions assigned to this category name. When omitted, all categories \
                     are included."
    )]
    from_category: Option<String>,
    /// Output serialization format (`json` by default)
    #[clap(
        long,
        arg_enum,
        default_value_t = OutputFormat::Json,
        long_help = EXPORT_FORMAT_LONG_HELP
    )]
    format: OutputFormat,
}

/// Output encoding for export subcommands.
#[derive(ArgEnum, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    /// Pretty-printed JSON (default)
    Json,
}

#[cfg(feature = "experimental")]
#[derive(Subcommand)]
enum CreateTarget {
    /// Create a SEPA bank transfer from JSON (stdin or file)
    #[clap(name = "bank-transfer")]
    BankTransfer(BankTransferArgs),
}

#[cfg(feature = "experimental")]
#[derive(Args)]
struct BankTransferArgs {
    /// Path to JSON parameters, or `-` / omit for stdin
    file: Option<PathBuf>,
}

fn parse_naive_date(label: &str, s: &str) -> Result<NaiveDate, String> {
    NaiveDate::from_str(s).map_err(|e| format!("{label}: invalid date `{s}`: {e}"))
}

fn write_json_pretty_stdout<T: Serialize>(
    value: &T,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, value)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

/// Serialize export items for CLI JSON: drop `icon` unless `--include-icon-data` was passed.
fn export_json_value_without_icons<T: Serialize>(
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

#[cfg(feature = "experimental")]
fn read_json_input(file: Option<PathBuf>) -> io::Result<String> {
    fn empty_stdin_error() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "expected JSON on stdin (redirect a file or pass a path); stdin was empty",
        )
    }

    match file {
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            if buf.trim().is_empty() {
                return Err(empty_stdin_error());
            }
            Ok(buf)
        }
        Some(p) if p.as_os_str() == "-" => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            if buf.trim().is_empty() {
                return Err(empty_stdin_error());
            }
            Ok(buf)
        }
        Some(p) => std::fs::read_to_string(p),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cli.command {
        Cmd::Export { target } => match target {
            ExportTarget::Accounts(args) => {
                let accounts = moneymoney::export_accounts()?;
                let accounts = if args.include_group_accounts {
                    accounts
                } else {
                    accounts.into_iter().filter(|a| !a.group).collect()
                };
                match args.format {
                    OutputFormat::Json => {
                        let json =
                            export_json_value_without_icons(&accounts, args.include_icon_data)?;
                        write_json_pretty_stdout(&json)?;
                    }
                }
            }
            ExportTarget::Categories(args) => {
                let categories = moneymoney::export_categories()?;
                let categories = if args.include_group_categories {
                    categories
                } else {
                    categories.into_iter().filter(|c| !c.group).collect()
                };
                match args.format {
                    OutputFormat::Json => {
                        let json =
                            export_json_value_without_icons(&categories, args.include_icon_data)?;
                        write_json_pretty_stdout(&json)?;
                    }
                }
            }
            ExportTarget::Transactions(args) => {
                let from_date = parse_naive_date("--from-date", &args.from_date)?;
                let mut params = ExportTransactionsParams::new(from_date);
                if let Some(ref s) = args.to_date {
                    params.to_date = Some(parse_naive_date("--to-date", s)?);
                }
                if let Some(a) = args.from_account {
                    params.from_account = Some(a);
                }
                if let Some(c) = args.from_category {
                    params.from_category = Some(c);
                }
                let response = moneymoney::export_transactions(params)?;
                match args.format {
                    OutputFormat::Json => write_json_pretty_stdout(&response)?,
                }
            }
        },
        #[cfg(feature = "experimental")]
        Cmd::Create { target } => match target {
            CreateTarget::BankTransfer(args) => {
                use moneymoney::create_bank_transfer::CreateBankTransferParams;

                let raw = read_json_input(args.file)?;
                let params: CreateBankTransferParams = serde_json::from_str(raw.trim())
                    .map_err(|e| format!("invalid JSON for bank transfer: {e}"))?;
                let result = moneymoney::create_bank_transfer(params)?;
                write_json_pretty_stdout(&result)?;
            }
        },
    }
    Ok(())
}

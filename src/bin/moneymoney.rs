//! Command-line interface to MoneyMoney (macOS).
//!
//! Build with `--features experimental` to enable `create bank-transfer`.

#[cfg(feature = "experimental")]
use std::io::Read;
use std::io::{self, Write};
#[cfg(feature = "experimental")]
use std::path::PathBuf;
use std::str::FromStr;

use chrono::NaiveDate;
use clap::{ArgEnum, Args, Parser, Subcommand};
use moneymoney::export_transactions::ExportTransactionsParams;
use serde::Serialize;

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
    /// Export accounts (balances, metadata) as JSON
    Accounts(ExportAccountsArgs),
    /// Export transactions as JSON (or other formats)
    Transactions(ExportTransactionsArgs),
}

#[derive(Args)]
struct ExportAccountsArgs {
    /// Output encoding
    #[clap(long, arg_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
    /// Include per-account icon bytes in JSON (large; omitted by default)
    #[clap(long = "include-icon-data")]
    include_icon_data: bool,
}

#[derive(Args)]
struct ExportTransactionsArgs {
    /// Inclusive start date (YYYY-MM-DD)
    #[clap(long = "from-date", value_name = "YYYY-MM-DD")]
    from_date: String,
    /// Inclusive end date (YYYY-MM-DD)
    #[clap(long = "to-date", value_name = "YYYY-MM-DD")]
    to_date: Option<String>,
    /// Filter by account UUID or IBAN
    #[clap(long = "from-account")]
    from_account: Option<String>,
    /// Filter by category name
    #[clap(long = "from-category")]
    from_category: Option<String>,
    /// Output encoding
    #[clap(long, arg_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
}

#[derive(ArgEnum, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
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

/// Serialize accounts for CLI JSON: drop `icon` unless `--include-icon-data` was passed.
fn export_accounts_json_value(
    accounts: &[moneymoney::export_accounts::MoneymoneyAccount],
    include_icon_data: bool,
) -> Result<serde_json::Value, serde_json::Error> {
    let mut v = serde_json::to_value(accounts)?;
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
                match args.format {
                    OutputFormat::Json => {
                        let json = export_accounts_json_value(&accounts, args.include_icon_data)?;
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

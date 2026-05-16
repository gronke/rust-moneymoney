//! Unit tests for the `moneymoney` CLI binary.
//!
//! Pure tests with no MoneyMoney dependency — they exercise argv parsing
//! (via `Cli::try_parse_from`) and the helper functions defined in the
//! sibling `moneymoney.rs`. Reachable here through `mod tests;` declared
//! at the bottom of that file.

use chrono::NaiveDate;
use clap::Parser;

use super::{export_json_value_without_icons, Cli, Cmd, ExportTarget};

#[cfg(feature = "experimental")]
use super::CreateTarget;

#[test]
fn export_transactions_rejects_invalid_date_at_parse_time() {
    let result = Cli::try_parse_from([
        "moneymoney",
        "export",
        "transactions",
        "--from-date",
        "not-a-date",
    ]);
    let err = match result {
        Ok(_) => panic!("expected parse error for invalid --from-date"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("not-a-date"), "got: {msg}");
    assert!(msg.contains("--from-date"), "got: {msg}");
}

#[test]
fn icons_stripped_by_default() {
    let items = vec![serde_json::json!({"name": "a", "icon": "BINARY"})];
    let v = export_json_value_without_icons(&items, false).unwrap();
    assert!(v[0].get("icon").is_none());
    assert_eq!(v[0]["name"], "a");
}

#[test]
fn icons_retained_when_requested() {
    let items = vec![serde_json::json!({"name": "a", "icon": "BINARY"})];
    let v = export_json_value_without_icons(&items, true).unwrap();
    assert_eq!(v[0]["icon"], "BINARY");
}

#[test]
fn parses_export_accounts_default() {
    let cli = Cli::try_parse_from(["moneymoney", "export", "accounts"]).unwrap();
    let Cmd::Export {
        target: ExportTarget::Accounts(args),
    } = cli.command
    else {
        panic!("expected Export::Accounts");
    };
    assert!(!args.include_icon_data);
    assert!(!args.include_group_accounts);
}

#[test]
fn parses_export_accounts_include_flags() {
    let cli = Cli::try_parse_from([
        "moneymoney",
        "export",
        "accounts",
        "--include-icon-data",
        "--include-group-accounts",
    ])
    .unwrap();
    let Cmd::Export {
        target: ExportTarget::Accounts(args),
    } = cli.command
    else {
        panic!("expected Export::Accounts");
    };
    assert!(args.include_icon_data);
    assert!(args.include_group_accounts);
}

#[test]
fn parses_export_transactions_all_filters() {
    let cli = Cli::try_parse_from([
        "moneymoney",
        "export",
        "transactions",
        "--from-date",
        "2026-01-01",
        "--to-date",
        "2026-12-31",
        "--from-account",
        "DE89370400440532013000",
        "--from-category",
        "Groceries",
    ])
    .unwrap();
    let Cmd::Export {
        target: ExportTarget::Transactions(args),
    } = cli.command
    else {
        panic!("expected Export::Transactions");
    };
    assert_eq!(args.from_date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
    assert_eq!(args.to_date, NaiveDate::from_ymd_opt(2026, 12, 31));
    assert_eq!(args.from_account.as_deref(), Some("DE89370400440532013000"));
    assert_eq!(args.from_category.as_deref(), Some("Groceries"));
}

#[test]
fn export_transactions_requires_from_date() {
    let result = Cli::try_parse_from(["moneymoney", "export", "transactions"]);
    let err = match result {
        Ok(_) => panic!("expected parse error for missing --from-date"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("--from-date"), "got: {msg}");
}

#[test]
fn unknown_subcommand_is_rejected() {
    let result = Cli::try_parse_from(["moneymoney", "expert"]);
    let err = match result {
        Ok(_) => panic!("expected parse error for unknown subcommand"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("expert"), "got: {msg}");
}

#[cfg(feature = "experimental")]
#[test]
fn parses_create_bank_transfer_with_file() {
    let cli =
        Cli::try_parse_from(["moneymoney", "create", "bank-transfer", "/tmp/params.json"]).unwrap();
    let Cmd::Create {
        target: CreateTarget::BankTransfer(args),
    } = cli.command
    else {
        panic!("expected Create::BankTransfer");
    };
    assert_eq!(args.file.as_deref(), Some(std::path::Path::new("/tmp/params.json")));
}

#[cfg(feature = "experimental")]
#[test]
fn parses_create_bank_transfer_with_stdin_marker() {
    let cli = Cli::try_parse_from(["moneymoney", "create", "bank-transfer", "-"]).unwrap();
    let Cmd::Create {
        target: CreateTarget::BankTransfer(args),
    } = cli.command
    else {
        panic!("expected Create::BankTransfer");
    };
    assert_eq!(args.file.as_deref(), Some(std::path::Path::new("-")));
}

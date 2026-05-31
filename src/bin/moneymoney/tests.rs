//! Unit tests for the `moneymoney` CLI binary.
//!
//! Pure tests with no MoneyMoney dependency — they exercise argv parsing
//! (via `Cli::try_parse_from`) and the helper functions inside each
//! per-verb module.

use chrono::NaiveDate;
use clap::Parser;

use super::add::{self, AddTarget, AddTransactionArgs};
use super::batch::BatchOptions;
use super::export::{self, ExportTarget};
use super::set::{self, CheckmarkState, SetTarget, SetTransactionArgs};
use super::{Cli, Cmd};

#[cfg(feature = "experimental")]
use super::create::CreateTarget;

// -- helpers --------------------------------------------------------------

fn empty_batch() -> BatchOptions {
    BatchOptions {
        files: Vec::new(),
        dry_run: false,
        skip: 0,
        skip_error: false,
        skip_duplicates: false,
    }
}

// -- export ---------------------------------------------------------------

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
    let v = export::export_json_value_without_icons(&items, false).unwrap();
    assert!(v[0].get("icon").is_none());
    assert_eq!(v[0]["name"], "a");
}

#[test]
fn icons_retained_when_requested() {
    let items = vec![serde_json::json!({"name": "a", "icon": "BINARY"})];
    let v = export::export_json_value_without_icons(&items, true).unwrap();
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
    assert!(!args.options.include_icon_data);
    assert!(!args.options.include_groups);
}

#[test]
fn parses_export_accounts_include_flags() {
    let cli = Cli::try_parse_from([
        "moneymoney",
        "export",
        "accounts",
        "--include-icon-data",
        "--include-groups",
    ])
    .unwrap();
    let Cmd::Export {
        target: ExportTarget::Accounts(args),
    } = cli.command
    else {
        panic!("expected Export::Accounts");
    };
    assert!(args.options.include_icon_data);
    assert!(args.options.include_groups);
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
    assert_eq!(args.account.from_account.as_deref(), Some("DE89370400440532013000"));
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
fn parses_export_portfolio_no_filters() {
    let cli = Cli::try_parse_from(["moneymoney", "export", "portfolio"]).unwrap();
    let Cmd::Export {
        target: ExportTarget::Portfolio(args),
    } = cli.command
    else {
        panic!("expected Export::Portfolio");
    };
    assert!(args.account.from_account.is_none());
    assert!(args.from_asset_class.is_none());
}

#[test]
fn parses_export_portfolio_with_filters() {
    let cli = Cli::try_parse_from([
        "moneymoney",
        "export",
        "portfolio",
        "--from-account",
        "DE89370400440532013000",
        "--from-asset-class",
        "Aktien",
        "--format",
        "json",
    ])
    .unwrap();
    let Cmd::Export {
        target: ExportTarget::Portfolio(args),
    } = cli.command
    else {
        panic!("expected Export::Portfolio");
    };
    assert_eq!(args.account.from_account.as_deref(), Some("DE89370400440532013000"));
    assert_eq!(args.from_asset_class.as_deref(), Some("Aktien"));
}

// -- global flags ---------------------------------------------------------

#[test]
fn parses_global_no_color_after_subcommand() {
    let cli = Cli::try_parse_from(["moneymoney", "export", "accounts", "--no-color"]).unwrap();
    assert!(cli.no_color);
}

#[test]
fn parses_global_format_at_top_level() {
    let cli =
        Cli::try_parse_from(["moneymoney", "--format", "json", "export", "accounts"]).unwrap();
    matches!(cli.format, super::output::OutputFormat::Json)
        .then_some(())
        .expect("expected --format json");
}

// -- add transaction ------------------------------------------------------

#[test]
fn parses_add_transaction_all_required() {
    let cli = Cli::try_parse_from([
        "moneymoney",
        "add",
        "transaction",
        "--account",
        "test-cash",
        "--date",
        "2026-05-17",
        "--name",
        "Coffee Shop",
        "--amount",
        "-3.50",
    ])
    .unwrap();
    let Cmd::Add {
        target: AddTarget::Transaction(args),
    } = cli.command
    else {
        panic!("expected Add::Transaction");
    };
    assert_eq!(args.account.as_deref(), Some("test-cash"));
    assert_eq!(args.date, NaiveDate::from_ymd_opt(2026, 5, 17));
    assert_eq!(args.name.as_deref(), Some("Coffee Shop"));
    assert!(args.to.is_none());
    assert!(args.from.is_none());
    assert_eq!(args.amount, Some(-3.50));
    assert!(args.purpose.is_none());
    assert!(args.category.is_none());
    assert!(args.batch.files.is_empty());
    assert!(!args.batch.dry_run);
}

#[test]
fn parses_add_transaction_with_to_alias() {
    let cli = Cli::try_parse_from([
        "moneymoney",
        "add",
        "transaction",
        "--account",
        "test-cash",
        "--date",
        "2026-05-17",
        "--to",
        "Coffee Shop",
        "--amount",
        "-3.50",
    ])
    .unwrap();
    let Cmd::Add {
        target: AddTarget::Transaction(args),
    } = cli.command
    else {
        panic!("expected Add::Transaction");
    };
    assert!(args.name.is_none());
    assert_eq!(args.to.as_deref(), Some("Coffee Shop"));
    assert!(args.from.is_none());
}

#[test]
fn add_transaction_rejects_mixed_name_aliases() {
    let result = Cli::try_parse_from([
        "moneymoney",
        "add",
        "transaction",
        "--account",
        "test-cash",
        "--date",
        "2026-05-17",
        "--name",
        "X",
        "--to",
        "Y",
        "--amount",
        "1.00",
    ]);
    let err = match result {
        Ok(_) => panic!("expected parse error for --name + --to mix"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("--name"), "got: {msg}");
    assert!(msg.contains("--to"), "got: {msg}");
}

#[test]
fn add_transaction_rejects_to_plus_from() {
    let result = Cli::try_parse_from([
        "moneymoney",
        "add",
        "transaction",
        "--account",
        "test-cash",
        "--date",
        "2026-05-17",
        "--to",
        "X",
        "--from",
        "Y",
        "--amount",
        "1.00",
    ]);
    let err = match result {
        Ok(_) => panic!("expected parse error for --to + --from mix"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("--to"), "got: {msg}");
    assert!(msg.contains("--from"), "got: {msg}");
}

#[test]
fn parses_add_transaction_with_from_alias() {
    let cli = Cli::try_parse_from([
        "moneymoney",
        "add",
        "transaction",
        "--account",
        "test-cash",
        "--date",
        "2026-05-17",
        "--from",
        "Employer GmbH",
        "--amount",
        "2000.00",
    ])
    .unwrap();
    let Cmd::Add {
        target: AddTarget::Transaction(args),
    } = cli.command
    else {
        panic!("expected Add::Transaction");
    };
    assert!(args.name.is_none());
    assert!(args.to.is_none());
    assert_eq!(args.from.as_deref(), Some("Employer GmbH"));
    assert_eq!(args.amount, Some(2000.00));
}

#[test]
fn add_transaction_rejects_invalid_date() {
    let result = Cli::try_parse_from([
        "moneymoney",
        "add",
        "transaction",
        "--account",
        "test-cash",
        "--date",
        "not-a-date",
        "--to",
        "Coffee Shop",
        "--amount",
        "-3.50",
    ]);
    let err = match result {
        Ok(_) => panic!("expected parse error for invalid --date"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("not-a-date"), "got: {msg}");
    assert!(msg.contains("--date"), "got: {msg}");
}

#[test]
fn add_transaction_bare_invocation_is_rejected_by_resolver() {
    let args = AddTransactionArgs {
        account: None,
        date: None,
        name: None,
        to: None,
        from: None,
        amount: None,
        purpose: None,
        category: None,
        batch: empty_batch(),
    };
    // build_from_flags sees no fields -> returns Ok(None); the real rejection
    // happens in collect_batch (no flag, no files).
    let flag_built = add::build_from_flags(&args).unwrap();
    assert!(flag_built.is_none());
    let err = super::batch::collect_batch::<moneymoney::add_transaction::AddTransactionParams>(
        flag_built,
        &args.batch,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("provide JSON"), "got: {msg}");
}

#[test]
fn add_transaction_resolve_requires_account_when_partial_flags() {
    let args = AddTransactionArgs {
        account: None,
        date: NaiveDate::from_ymd_opt(2026, 5, 17),
        name: Some("Payee".into()),
        to: None,
        from: None,
        amount: Some(1.0),
        purpose: None,
        category: None,
        batch: empty_batch(),
    };
    let err = add::build_from_flags(&args).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("--account"), "got: {msg}");
}

#[test]
fn parses_add_transaction_with_positional_file() {
    let cli =
        Cli::try_parse_from(["moneymoney", "add", "transaction", "/tmp/params.json"]).unwrap();
    let Cmd::Add {
        target: AddTarget::Transaction(args),
    } = cli.command
    else {
        panic!("expected Add::Transaction");
    };
    assert_eq!(args.batch.files.len(), 1);
    assert_eq!(args.batch.files[0].as_path(), std::path::Path::new("/tmp/params.json"));
    assert!(args.account.is_none());
}

#[test]
fn parses_add_transaction_with_multiple_files_and_stdin() {
    let cli =
        Cli::try_parse_from(["moneymoney", "add", "transaction", "a.json", "-", "b.json"]).unwrap();
    let Cmd::Add {
        target: AddTarget::Transaction(args),
    } = cli.command
    else {
        panic!("expected Add::Transaction");
    };
    assert_eq!(args.batch.files.len(), 3);
    assert_eq!(args.batch.files[1].as_os_str(), "-");
}

#[test]
fn add_transaction_rejects_flags_plus_files() {
    let args = AddTransactionArgs {
        account: Some("test-cash".into()),
        date: NaiveDate::from_ymd_opt(2026, 5, 17),
        name: Some("Payee".into()),
        to: None,
        from: None,
        amount: Some(1.0),
        purpose: None,
        category: None,
        batch: BatchOptions {
            files: vec![std::path::PathBuf::from("/tmp/x.json")],
            ..empty_batch()
        },
    };
    let flag_built = add::build_from_flags(&args).unwrap();
    assert!(flag_built.is_some());
    let err = super::batch::collect_batch::<moneymoney::add_transaction::AddTransactionParams>(
        flag_built,
        &args.batch,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("mutually exclusive"), "got: {msg}");
}

#[test]
fn parses_add_transaction_batch_flags() {
    let cli = Cli::try_parse_from([
        "moneymoney",
        "add",
        "transaction",
        "--dry-run",
        "--skip",
        "3",
        "--skip-error",
        "--skip-duplicates",
        "batch.json",
    ])
    .unwrap();
    let Cmd::Add {
        target: AddTarget::Transaction(args),
    } = cli.command
    else {
        panic!("expected Add::Transaction");
    };
    assert!(args.batch.dry_run);
    assert_eq!(args.batch.skip, 3);
    assert!(args.batch.skip_error);
    assert!(args.batch.skip_duplicates);
}

// -- set transaction ------------------------------------------------------

#[test]
fn parses_set_transaction_with_modifiers() {
    let cli = Cli::try_parse_from([
        "moneymoney",
        "set",
        "transaction",
        "--id",
        "421337",
        "--checkmark",
        "on",
        "--comment",
        "reviewed",
    ])
    .unwrap();
    let Cmd::Set {
        target: SetTarget::Transaction(args),
    } = cli.command
    else {
        panic!("expected Set::Transaction");
    };
    assert_eq!(args.id, Some(421337));
    assert_eq!(args.checkmark, Some(CheckmarkState::On));
    assert_eq!(args.comment.as_deref(), Some("reviewed"));
    assert!(args.category.is_none());
}

#[test]
fn set_transaction_requires_a_modifier() {
    let args = SetTransactionArgs {
        id: Some(42),
        checkmark: None,
        category: None,
        comment: None,
        batch: empty_batch(),
    };
    let err = set::build_from_flags(&args).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("--checkmark"), "got: {msg}");
    assert!(msg.contains("--category"), "got: {msg}");
    assert!(msg.contains("--comment"), "got: {msg}");
}

#[test]
fn set_transaction_resolve_requires_id_when_partial_flags() {
    let args = SetTransactionArgs {
        id: None,
        checkmark: Some(CheckmarkState::On),
        category: None,
        comment: None,
        batch: empty_batch(),
    };
    let err = set::build_from_flags(&args).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("--id"), "got: {msg}");
}

#[test]
fn set_transaction_rejects_invalid_checkmark() {
    let result = Cli::try_parse_from([
        "moneymoney",
        "set",
        "transaction",
        "--id",
        "1",
        "--checkmark",
        "maybe",
    ]);
    let err = match result {
        Ok(_) => panic!("expected parse error for invalid --checkmark"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("maybe"), "got: {msg}");
}

#[test]
fn set_transaction_rejects_skip_duplicates() {
    let args = SetTransactionArgs {
        id: Some(1),
        checkmark: Some(CheckmarkState::On),
        category: None,
        comment: None,
        batch: BatchOptions {
            skip_duplicates: true,
            ..empty_batch()
        },
    };
    let err = super::batch::reject_skip_duplicates(&args.batch, "set transaction").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("--skip-duplicates"), "got: {msg}");
    assert!(msg.contains("add transaction"), "got: {msg}");
}

// -- unknown / create -----------------------------------------------------

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
    assert_eq!(args.batch.files.len(), 1);
    assert_eq!(args.batch.files[0].as_path(), std::path::Path::new("/tmp/params.json"));
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
    assert_eq!(args.batch.files.len(), 1);
    assert_eq!(args.batch.files[0].as_os_str(), "-");
}

#[cfg(feature = "experimental")]
#[test]
fn parses_create_direct_debit_with_file() {
    let cli =
        Cli::try_parse_from(["moneymoney", "create", "direct-debit", "/tmp/params.json"]).unwrap();
    let Cmd::Create {
        target: CreateTarget::DirectDebit(args),
    } = cli.command
    else {
        panic!("expected Create::DirectDebit");
    };
    assert_eq!(args.batch.files.len(), 1);
    assert_eq!(args.batch.files[0].as_path(), std::path::Path::new("/tmp/params.json"));
}

#[cfg(feature = "experimental")]
#[test]
fn create_bank_transfer_rejects_skip_duplicates() {
    let batch = BatchOptions {
        skip_duplicates: true,
        ..empty_batch()
    };
    let err = super::batch::reject_skip_duplicates(&batch, "create bank-transfer").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("--skip-duplicates"), "got: {msg}");
}

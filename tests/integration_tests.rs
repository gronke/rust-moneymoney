//! Integration tests for the moneymoney library.
//!
//! These tests require MoneyMoney to be running and are marked with `#[ignore]`
//! by default. Run them with: `cargo test -- --ignored`
//!
//! The `create bank-transfer` CLI test also needs `--features experimental`.

use chrono::NaiveDate;
use moneymoney::export_accounts::MoneymoneyAccountType;
use moneymoney::export_transactions::ExportTransactionsParams;
use moneymoney::{export_accounts, export_categories, export_transactions};

/// Test the complete workflow: accounts -> categories -> transactions
#[test]
#[ignore]
fn test_complete_workflow() {
    // 1. Export accounts
    let accounts = export_accounts::export_accounts().expect("Failed to export accounts");
    assert!(!accounts.is_empty(), "Should have at least one account");

    // 2. Export categories
    let categories = export_categories::export_categories().expect("Failed to export categories");
    assert!(!categories.is_empty(), "Should have at least one category");

    // 3. Export transactions
    let from_date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Valid date");
    let params = ExportTransactionsParams::new(from_date);
    let response =
        export_transactions::export_transactions(params).expect("Failed to export transactions");

    // Verify response structure
    assert!(!response.creator.is_empty(), "Creator should not be empty");
    // Note: transactions may be empty if no transactions exist in the date range
}

/// Test account hierarchy and grouping
#[test]
#[ignore]
fn test_account_hierarchy() {
    let accounts = export_accounts::export_accounts().expect("Failed to export accounts");

    // Check for account groups
    let groups: Vec<_> = accounts.iter().filter(|a| a.group).collect();
    let non_groups: Vec<_> = accounts.iter().filter(|a| !a.group).collect();

    println!(
        "Found {} account groups and {} regular accounts",
        groups.len(),
        non_groups.len()
    );

    // Verify hierarchy via indentation
    for account in &accounts {
        assert!(account.indentation < 10, "Indentation should be reasonable");
    }

    // Verify all non-group accounts have valid UUIDs
    for account in &non_groups {
        assert!(!account.uuid.is_nil(), "Account UUID should not be nil");
    }
}

/// Test category hierarchy and budgets
#[test]
#[ignore]
fn test_category_hierarchy() {
    let categories = export_categories::export_categories().expect("Failed to export categories");

    let groups: Vec<_> = categories.iter().filter(|c| c.group).collect();
    let non_groups: Vec<_> = categories.iter().filter(|c| !c.group).collect();

    println!(
        "Found {} category groups and {} regular categories",
        groups.len(),
        non_groups.len()
    );

    // Check for budgets
    let with_budget: Vec<_> = non_groups.iter().filter(|c| c.budget.is_some()).collect();
    println!("Found {} categories with budgets", with_budget.len());

    // Verify budget data if present
    for category in with_budget {
        if let Some(budget) = &category.budget {
            assert!(budget.amount >= 0.0, "Budget amount should be non-negative");
            assert!(!budget.period.is_empty(), "Budget period should not be empty");
        }
    }
}

/// Test transaction filtering by date range
#[test]
#[ignore]
fn test_transaction_date_filtering() {
    let from_date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Valid date");
    let to_date = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Valid date");

    let params = ExportTransactionsParams::new(from_date).to_date(to_date);
    let response =
        export_transactions::export_transactions(params).expect("Failed to export transactions");

    // Verify all transactions are within the date range
    for transaction in &response.transactions {
        let booking_date = transaction.booking_date.date_naive();
        assert!(
            booking_date >= from_date && booking_date <= to_date,
            "Transaction booking date {} should be within range {} to {}",
            booking_date,
            from_date,
            to_date
        );
    }

    println!("Found {} transactions in date range", response.transactions.len());
}

/// Test transaction filtering by account
#[test]
#[ignore]
fn test_transaction_account_filtering() {
    // First, get all accounts
    let accounts = export_accounts::export_accounts().expect("Failed to export accounts");

    // Find a non-group account
    if let Some(account) = accounts.iter().find(|a| !a.group) {
        let from_date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Valid date");
        let params =
            ExportTransactionsParams::new(from_date).from_account(account.uuid.to_string());

        let response = export_transactions::export_transactions(params)
            .expect("Failed to export transactions");

        // Verify all transactions belong to the specified account
        for transaction in &response.transactions {
            assert_eq!(
                transaction.account_uuid, account.uuid,
                "Transaction should belong to the filtered account"
            );
        }

        println!(
            "Found {} transactions for account '{}'",
            response.transactions.len(),
            account.name
        );
    } else {
        panic!("No non-group accounts found for testing");
    }
}

/// Test transaction filtering by category
#[test]
#[ignore]
fn test_transaction_category_filtering() {
    // First, get all categories
    let categories = export_categories::export_categories().expect("Failed to export categories");

    // Find a non-group category
    if let Some(category) = categories.iter().find(|c| !c.group) {
        let from_date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Valid date");
        let params = ExportTransactionsParams::new(from_date).from_category(category.name.clone());

        let response = export_transactions::export_transactions(params)
            .expect("Failed to export transactions");

        // Verify all transactions belong to the specified category
        for transaction in &response.transactions {
            assert_eq!(
                transaction.category_uuid, category.uuid,
                "Transaction should belong to the filtered category"
            );
        }

        println!(
            "Found {} transactions for category '{}'",
            response.transactions.len(),
            category.name
        );
    } else {
        panic!("No non-group categories found for testing");
    }
}

/// Test transaction data validity
#[test]
#[ignore]
fn test_transaction_data_validity() {
    let from_date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Valid date");
    let params = ExportTransactionsParams::new(from_date);
    let response =
        export_transactions::export_transactions(params).expect("Failed to export transactions");

    for transaction in &response.transactions {
        // Verify required fields are populated
        assert!(transaction.id > 0, "Transaction ID should be positive");
        assert!(!transaction.name.is_empty(), "Transaction name should not be empty");
        assert!(!transaction.currency.is_empty(), "Currency should not be empty");
        assert!(!transaction.account_uuid.is_nil(), "Account UUID should not be nil");
        assert!(!transaction.category_uuid.is_nil(), "Category UUID should not be nil");

        // Verify date fields are reasonable
        assert!(
            transaction.value_date >= transaction.booking_date
                || (transaction.value_date.date_naive() - transaction.booking_date.date_naive())
                    .num_days()
                    .abs()
                    <= 7,
            "Value date and booking date should be close"
        );
    }

    println!("Validated {} transactions", response.transactions.len());
}

/// Test account balance consistency
#[test]
#[ignore]
fn test_account_balance_consistency() {
    let accounts = export_accounts::export_accounts().expect("Failed to export accounts");

    for account in accounts.iter().filter(|a| !a.group) {
        let Some(balance) = &account.balance else {
            // Non-group accounts may still have an empty balance array (e.g., a freshly
            // added account that hasn't synced yet). Skip — there's nothing to verify.
            continue;
        };

        assert_eq!(
            balance.currency.code(),
            account.currency,
            "Balance currency should match account currency for account '{}'",
            account.name
        );

        println!("Account '{}': {} {}", account.name, balance.amount, balance.currency.code());
    }
}

/// Test that the default Uncategorized category exists and has valid structure
#[test]
#[ignore]
fn test_uncategorized_category_exists() {
    let categories = export_categories::export_categories().expect("Failed to export categories");

    // MoneyMoney always has an "Uncategorized" category
    let uncategorized = categories
        .iter()
        .find(|c| c.name == "Uncategorized")
        .expect("Uncategorized category should exist");

    // Verify structure
    assert!(!uncategorized.uuid.is_nil(), "UUID should not be nil");
    assert!(!uncategorized.group, "Uncategorized should not be a group");
    assert!(uncategorized.default, "Uncategorized should be the default category");
    assert_eq!(uncategorized.indentation, 0, "Uncategorized should be at root level");

    println!(
        "Uncategorized category: uuid={}, currency={}",
        uncategorized.uuid,
        uncategorized.currency.code()
    );
}

/// Test combined filtering (date + account + category)
#[test]
#[ignore]
fn test_combined_filtering() {
    // Get accounts and categories
    let accounts = export_accounts::export_accounts().expect("Failed to export accounts");
    let categories = export_categories::export_categories().expect("Failed to export categories");

    if let (Some(account), Some(category)) =
        (accounts.iter().find(|a| !a.group), categories.iter().find(|c| !c.group))
    {
        let from_date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Valid date");
        let to_date = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Valid date");

        let params = ExportTransactionsParams::new(from_date)
            .to_date(to_date)
            .from_account(account.uuid.to_string())
            .from_category(category.name.clone());

        let response = export_transactions::export_transactions(params)
            .expect("Failed to export transactions");

        // Verify all filters are applied
        for transaction in &response.transactions {
            let booking_date = transaction.booking_date.date_naive();
            assert!(booking_date >= from_date && booking_date <= to_date);
            assert_eq!(transaction.account_uuid, account.uuid);
            assert_eq!(transaction.category_uuid, category.uuid);
        }

        println!("Found {} transactions matching all filters", response.transactions.len());
    }
}

/// Verify that every seeded test account deserializes as the expected typed
/// variant, and that no `test-` account falls through to `Custom`.
///
/// Run `scripts/create_test_accounts.sh` first to seed the accounts.
#[test]
#[ignore]
fn test_seeded_accounts_recognize_all_types() {
    let accounts = export_accounts::export_accounts().expect("export_accounts failed");

    let expected: &[(&str, MoneymoneyAccountType)] = &[
        ("test-cash", MoneymoneyAccountType::Cash),
        ("test-giro", MoneymoneyAccountType::Giro),
        ("test-savings", MoneymoneyAccountType::Savings),
        ("test-fixed-term", MoneymoneyAccountType::FixedTermDeposit),
        ("test-loan", MoneymoneyAccountType::Loan),
        ("test-creditcard", MoneymoneyAccountType::CreditCard),
    ];

    for (name, expected_type) in expected {
        let acc = accounts
            .iter()
            .find(|a| &a.name == name)
            .unwrap_or_else(|| {
                panic!("missing seeded account: {name} (run scripts/create_test_accounts.sh)")
            });
        assert_eq!(
            std::mem::discriminant(&acc.r#type),
            std::mem::discriminant(expected_type),
            "account '{name}' deserialized as {:?}, expected {:?}",
            acc.r#type,
            expected_type,
        );
    }

    for acc in accounts
        .iter()
        .filter(|a| a.name.starts_with("test-") && !a.group)
    {
        if let MoneymoneyAccountType::Custom(s) = &acc.r#type {
            panic!(
                "test account '{}' fell through to Custom({s:?}) — \
                 the deserializer does not recognize this type label",
                acc.name
            );
        }
    }
}

/// `create bank-transfer` CLI against the seeded offline account `test-cash`.
///
/// Requires MoneyMoney, the `experimental` feature, and `scripts/create_test_accounts.sh`.
/// Passes JSON via a temp file (not stdin) to the `moneymoney` binary.
/// Run: `cargo test --features experimental --test integration_tests test_cli_create_bank_transfer_from_test_cash_outbox -- --ignored`
#[cfg(feature = "experimental")]
mod create_bank_transfer_cli {
    use std::fs;
    use std::process::Command;

    use moneymoney::export_accounts;

    #[test]
    #[ignore]
    fn test_cli_create_bank_transfer_from_test_cash_outbox() {
        let accounts = export_accounts().expect("export_accounts");
        assert!(
            accounts.iter().any(|a| a.name == "test-cash" && !a.group),
            "missing offline account `test-cash` — run scripts/create_test_accounts.sh"
        );

        let json = br#"{"fromAccount":"test-cash","to":"moneymoney test payee","iban":"DE89370400440532013000","amount":0.01,"purpose":"CLI create bank-transfer integration test","into":"outbox"}"#;

        let path = std::env::temp_dir()
            .join(format!("moneymoney-cli-bank-transfer-test-{}.json", std::process::id()));
        fs::write(&path, json).expect("write temp JSON");

        let output = Command::new(env!("CARGO_BIN_EXE_moneymoney"))
            .args(["create", "bank-transfer"])
            .arg(&path)
            .output()
            .expect("run moneymoney create bank-transfer");

        let _ = fs::remove_file(&path);

        assert!(
            output.status.success(),
            "moneymoney create bank-transfer failed (exit={})\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

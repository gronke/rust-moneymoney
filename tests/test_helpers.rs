//! Test helper functions for integration tests.
//!
//! The required-accounts list is the single source of truth in
//! `src/test_utils.rs::REQUIRED_TEST_ACCOUNTS`. Run
//! `scripts/create_test_accounts.sh` to seed them.

use moneymoney::export_accounts;

const REQUIRED_TEST_ACCOUNTS: &[(&str, &str)] = &[
    ("test-cash", "Cash account"),
    ("test-giro", "Giro account"),
    ("test-savings", "Savings account"),
    ("test-fixed-term", "Fixed term deposit"),
    ("test-loan", "Loan account"),
    ("test-creditcard", "Credit card"),
];

pub fn ensure_test_accounts_exist() -> Result<(), String> {
    let accounts = export_accounts::export_accounts()
        .map_err(|e| format!("Failed to connect to MoneyMoney. Is it running? Error: {}", e))?;

    let test_accounts: Vec<_> = accounts
        .iter()
        .filter(|a| a.name.starts_with("test-"))
        .collect();

    let missing: Vec<&(&str, &str)> = REQUIRED_TEST_ACCOUNTS
        .iter()
        .filter(|(name, _)| !test_accounts.iter().any(|a| a.name == *name))
        .collect();

    if !missing.is_empty() {
        let missing_lines = missing
            .iter()
            .map(|(name, type_label)| format!("  [miss] {name} ({type_label})"))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "\n\nERROR: TEST ACCOUNTS MISSING\n\n\
            Integration tests require these offline accounts in MoneyMoney:\n\n\
            {missing_lines}\n\n\
            Run `scripts/create_test_accounts.sh` to create them automatically,\n\
            or create them manually via Account -> Add Account... -> Other -> Offline.\n\
            Tests only touch accounts whose name starts with `test-`.\n"
        ));
    }

    println!("OK: Found {} test accounts:", test_accounts.len());
    for account in &test_accounts {
        println!("   - {} ({})", account.name, account.currency);
    }

    Ok(())
}

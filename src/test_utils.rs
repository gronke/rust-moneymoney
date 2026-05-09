//! Test utilities for integration testing with MoneyMoney.
//!
//! This module provides helpers for writing integration tests that interact
//! with MoneyMoney. Enable the `test-utils` feature to use these utilities.
//!
//! # Test Account Convention
//!
//! All test accounts must be prefixed with `test-` to ensure they're easily
//! identifiable and won't conflict with real financial data.
//!
//! # Example
//!
//! ```rust,no_run
//! use moneymoney::test_utils::{TEST_ACCOUNT_PREFIX, ensure_test_accounts_exist, get_test_accounts};
//!
//! #[test]
//! fn test_with_moneymoney() {
//!     ensure_test_accounts_exist().expect("Test accounts required");
//!     let accounts = get_test_accounts().expect("Failed to get test accounts");
//!     assert!(accounts.len() >= 2);
//! }
//! ```

use crate::export_accounts::{self, MoneymoneyAccount};

/// Prefix used to identify test accounts in MoneyMoney.
///
/// All test accounts should be named with this prefix (e.g., "test-cash", "test-giro")
/// to ensure they're easily identifiable and isolated from real financial data.
pub const TEST_ACCOUNT_PREFIX: &str = "test-";

/// Required test accounts for full integration testing.
///
/// Each entry is `(name, type_label)` where `type_label` matches the string
/// MoneyMoney emits for that account's `type` field. These cover every account
/// type creatable via the offline-account UI; `Portfolio` and `Tagesgeld` are
/// not listed because MoneyMoney does not offer them as offline-creatable types.
pub const REQUIRED_TEST_ACCOUNTS: &[(&str, &str)] = &[
    ("test-cash", "Cash account"),
    ("test-giro", "Giro account"),
    ("test-savings", "Savings account"),
    ("test-fixed-term", "Fixed term deposit"),
    ("test-loan", "Loan account"),
    ("test-creditcard", "Credit card"),
];

/// Get all accounts that match the test account prefix.
///
/// Returns all accounts whose names start with `test-`.
///
/// # Errors
///
/// Returns an error if MoneyMoney is not running or communication fails.
pub fn get_test_accounts() -> Result<Vec<MoneymoneyAccount>, crate::Error> {
    let accounts = export_accounts::export_accounts()?;
    Ok(accounts
        .into_iter()
        .filter(|a| a.name.starts_with(TEST_ACCOUNT_PREFIX))
        .collect())
}

/// Ensure that required test accounts exist in MoneyMoney.
///
/// This function checks for the presence of test accounts and returns
/// a helpful error message with setup instructions if they're missing.
///
/// # Required Accounts
///
/// See [`REQUIRED_TEST_ACCOUNTS`] for the canonical list and their types.
///
/// # Errors
///
/// Returns a descriptive error if:
/// - MoneyMoney is not running
/// - No test accounts are found
/// - Some required test accounts are missing
pub fn ensure_test_accounts_exist() -> Result<Vec<MoneymoneyAccount>, String> {
    let accounts = export_accounts::export_accounts()
        .map_err(|e| format!("Failed to connect to MoneyMoney. Is it running? Error: {}", e))?;

    let test_accounts: Vec<_> = accounts
        .into_iter()
        .filter(|a| a.name.starts_with(TEST_ACCOUNT_PREFIX))
        .collect();

    if test_accounts.is_empty() {
        return Err(format!(
            "\n\nERROR: NO TEST ACCOUNTS FOUND\n\n\
            Integration tests require test accounts to be created manually.\n\
            This is a ONE-TIME setup (MoneyMoney's API doesn't support account creation).\n\n\
            Please create these offline accounts in MoneyMoney:\n\n\
            {}\n\n\
            How to create:\n\
               - Open MoneyMoney\n\
               - File -> New Account (Cmd-N)\n\
               - Select \"Offline Account\"\n\
               - Choose account type\n\
               - Enter name and currency (EUR)\n\
               - Click Create\n\n\
            After creating accounts, run the tests again.\n\
            Tests use only '{}' prefixed accounts and won't touch your real data.\n",
            REQUIRED_TEST_ACCOUNTS
                .iter()
                .enumerate()
                .map(|(i, (name, desc))| format!(
                    "{}. Account name: {}\n   Type: {}\n   Currency: EUR",
                    i + 1,
                    name,
                    desc
                ))
                .collect::<Vec<_>>()
                .join("\n\n"),
            TEST_ACCOUNT_PREFIX
        ));
    }

    // Check for specific required accounts
    let missing: Vec<_> = REQUIRED_TEST_ACCOUNTS
        .iter()
        .filter(|(name, _)| !test_accounts.iter().any(|a| a.name == *name))
        .collect();

    if !missing.is_empty() {
        return Err(format!(
            "\n\nWARNING: INCOMPLETE TEST SETUP\n\n\
            Found {} test account(s):\n{}\n\n\
            Missing accounts:\n{}\n\n\
            Please create the missing accounts in MoneyMoney (see above for instructions).\n",
            test_accounts.len(),
            test_accounts
                .iter()
                .map(|a| format!("  [ok]   {} ({})", a.name, a.currency))
                .collect::<Vec<_>>()
                .join("\n"),
            missing
                .iter()
                .map(|(name, desc)| format!("  [miss] {} ({})", name, desc))
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }

    Ok(test_accounts)
}

/// Find a test account by name.
///
/// # Arguments
///
/// * `name` - The account name (must start with `test-`)
///
/// # Returns
///
/// The account if found, or None if not found.
pub fn find_test_account(name: &str) -> Result<Option<MoneymoneyAccount>, crate::Error> {
    let accounts = get_test_accounts()?;
    Ok(accounts.into_iter().find(|a| a.name == name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_constant() {
        assert_eq!(TEST_ACCOUNT_PREFIX, "test-");
    }

    #[test]
    fn test_required_accounts_defined() {
        let names: Vec<&str> = REQUIRED_TEST_ACCOUNTS
            .iter()
            .map(|(name, _)| *name)
            .collect();
        for expected in [
            "test-cash",
            "test-giro",
            "test-savings",
            "test-fixed-term",
            "test-loan",
            "test-creditcard",
        ] {
            assert!(names.contains(&expected), "REQUIRED_TEST_ACCOUNTS missing {expected}");
        }
    }
}

//! # MoneyMoney Rust Interface
//!
//! A safe, ergonomic Rust interface to the [MoneyMoney](https://moneymoney-app.com/)
//! macOS application via AppleScript.
//!
//! ## Overview
//!
//! This library provides typed Rust bindings to MoneyMoney's AppleScript API, enabling
//! programmatic access to your financial data on macOS. All operations communicate with
//! the MoneyMoney application via OSA (Open Scripting Architecture).
//!
//! ## Requirements
//!
//! - **macOS**: MoneyMoney is a macOS-only application
//! - **MoneyMoney app**: Must be installed and running
//! - **Permissions**: Appropriate accessibility permissions for script execution
//!
//! ## Features
//!
//! - **Type-safe API**: All data structures use proper Rust types
//! - **Serde integration**: All types support serialization/deserialization
//! - **Zero unsafe code**: Pure safe Rust implementation
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use moneymoney::export_transactions::ExportTransactionsParams;
//! use chrono::NaiveDate;
//!
//! # fn main() -> Result<(), moneymoney::Error> {
//! // Export all accounts
//! let accounts = moneymoney::export_accounts()?;
//! for account in accounts {
//!     println!("{}: {} {}",
//!         account.name,
//!         account.balance.amount,
//!         account.balance.currency.code()
//!     );
//! }
//!
//! // Export transactions from a specific date
//! let params = ExportTransactionsParams::new(
//!     NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date")
//! );
//! let response = moneymoney::export_transactions(params)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Available Operations
//!
//! ### Export Operations
//! - [`export_accounts()`] - Export all accounts with balances and metadata
//! - [`export_categories()`] - Export all categories with budgets
//! - [`export_transactions()`] - Export transactions with flexible filtering
//! - [`export_portfolio()`] - Export securities and portfolio holdings
//!
//! ### Transaction Management
//! - [`add_transaction()`] - Add transactions to offline accounts
//! - [`set_transaction()`] - Modify existing transaction properties (checkmark, category, comment)
//!
//! ### Payment Operations (Experimental)
//! - [`create_bank_transfer()`] - Create SEPA bank transfers (requires `experimental` feature)
//! - [`create_direct_debit()`] - Create SEPA direct debit orders (requires `experimental` feature)
//!
//! ## Feature Flags
//!
//! - `experimental` - Enables experimental APIs like `create_bank_transfer` that may change
//!
//! ## MoneyMoney API Documentation
//!
//! For details on the underlying AppleScript API, see:
//! <https://moneymoney-app.com/api/>

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

mod methods;
pub use methods::*;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum MoneymoneyActions {
    ExportAccounts,
    ExportCategories,
    ExportTransactions(methods::export_transactions::ExportTransactionsParams),
    ExportPortfolio(methods::export_portfolio::ExportPortfolioParams),
    AddTransaction(methods::add_transaction::AddTransactionParams),
    SetTransaction(methods::set_transaction::SetTransactionParams),
    #[cfg(feature = "experimental")]
    CreateBankTransfer(methods::create_bank_transfer::CreateBankTransferParams),
    #[cfg(feature = "experimental")]
    CreateDirectDebit(methods::create_direct_debit::CreateDirectDebitParams),
}

impl MoneymoneyActions {
    fn method_name(&self) -> String {
        match self {
            MoneymoneyActions::ExportAccounts => "exportAccounts".to_string(),
            MoneymoneyActions::ExportCategories => "exportCategories".to_string(),
            MoneymoneyActions::ExportTransactions(_) => "exportTransactions".to_string(),
            MoneymoneyActions::ExportPortfolio(_) => "exportPortfolio".to_string(),
            MoneymoneyActions::AddTransaction(_) => "addTransaction".to_string(),
            MoneymoneyActions::SetTransaction(_) => "setTransaction".to_string(),
            #[cfg(feature = "experimental")]
            MoneymoneyActions::CreateBankTransfer(_) => "createBankTransfer".to_string(),
            #[cfg(feature = "experimental")]
            MoneymoneyActions::CreateDirectDebit(_) => "createDirectDebit".to_string(),
        }
    }
}

/// Errors that can occur when interacting with MoneyMoney.
///
/// This enum represents all possible error conditions that may arise
/// when communicating with the MoneyMoney application or processing its responses.
#[derive(Debug, Error)]
pub enum Error {
    /// An error occurred during OSA script execution.
    ///
    /// This typically indicates that:
    /// - MoneyMoney is not running
    /// - The script execution was denied
    /// - JavaScript/AppleScript syntax error
    #[error("OSA script execution failed: {0}")]
    OsaScript(#[from] osascript::Error),

    /// An error occurred while parsing the plist response from MoneyMoney.
    ///
    /// This usually indicates that MoneyMoney returned data in an unexpected format.
    #[error("Plist deserialization failed: {0}")]
    Plist(#[from] plist::Error),

    /// MoneyMoney returned an empty response when data was expected.
    ///
    /// This may occur if:
    /// - No data matches the query criteria
    /// - The operation succeeded but has no return value
    #[error("Received empty plist response from MoneyMoney")]
    EmptyPlist,

    /// An invalid currency code was encountered during parsing.
    ///
    /// This error contains the invalid currency code string that was received.
    #[error("Invalid currency code: {0}")]
    InvalidCurrency(String),
}

#[derive(Serialize, Deserialize)]
struct ScriptAction {
    method: String,
    args: MoneymoneyActions,
}

pub fn call_action(action: MoneymoneyActions) -> Result<Option<String>, osascript::Error> {
    let params = ScriptAction {
        method: action.method_name(),
        args: action,
    };
    let script = osascript::JavaScript::new(
        "
        if ($params.args) {
            $params.args['as'] = 'plist';
        }
        return Application('MoneyMoney')[$params.method]($params.args || []);
    ",
    );
    script.execute_with_params(&params)
}

/// Call a MoneyMoney action that doesn't return data (void operations).
///
/// Used for operations like `addTransaction` and `setTransaction` that modify
/// data but don't return a result.
pub fn call_action_void(action: MoneymoneyActions) -> Result<(), osascript::Error> {
    let params = ScriptAction {
        method: action.method_name(),
        args: action,
    };
    let script = osascript::JavaScript::new(
        "
        Application('MoneyMoney')[$params.method]($params.args || {});
        return true;
    ",
    );
    let _result: bool = script.execute_with_params(&params)?;
    Ok(())
}

pub fn call_action_plist<T>(action: MoneymoneyActions) -> Result<T, Error>
where
    T: DeserializeOwned + Serialize,
{
    let plist_response = call_action(action).map_err(Error::OsaScript)?;

    match plist_response {
        Some(v) => Ok(plist::from_bytes(v.as_bytes()).map_err(Error::Plist)?),
        None => Err(Error::EmptyPlist),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for Error type
    #[test]
    fn test_error_display_empty_plist() {
        let error = Error::EmptyPlist;
        assert_eq!(error.to_string(), "Received empty plist response from MoneyMoney");
    }

    #[test]
    fn test_error_display_invalid_currency() {
        let error = Error::InvalidCurrency("XYZ".to_string());
        assert_eq!(error.to_string(), "Invalid currency code: XYZ");
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }

    #[test]
    fn test_error_from_plist() {
        // Test that plist errors convert to Error::Plist variant
        let invalid_plist = b"invalid plist data";
        let result: Result<String, plist::Error> = plist::from_bytes(invalid_plist);

        if let Err(plist_error) = result {
            let error: Error = plist_error.into();
            assert!(matches!(error, Error::Plist(_)));
            assert!(error.to_string().contains("Plist deserialization failed"));
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = Error::EmptyPlist;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("EmptyPlist"));
    }

    // Unit tests for MoneymoneyActions
    #[test]
    fn test_action_method_names() {
        assert_eq!(MoneymoneyActions::ExportAccounts.method_name(), "exportAccounts");
        assert_eq!(MoneymoneyActions::ExportCategories.method_name(), "exportCategories");
    }

    #[test]
    fn test_export_transactions_action_method_name() {
        let params = methods::export_transactions::ExportTransactionsParams::new(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        let action = MoneymoneyActions::ExportTransactions(params);
        assert_eq!(action.method_name(), "exportTransactions");
    }

    #[cfg(feature = "experimental")]
    #[test]
    fn test_create_bank_transfer_action_method_name() {
        let params = methods::create_bank_transfer::CreateBankTransferParams {
            from_account: Some("test".to_string()),
            amount: Some(100.0),
            purpose: Some("Test".to_string()),
            ..Default::default()
        };
        let action = MoneymoneyActions::CreateBankTransfer(params);
        assert_eq!(action.method_name(), "createBankTransfer");
    }
}

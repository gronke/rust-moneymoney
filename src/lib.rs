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
//! use moneymoney::{export_accounts, export_transactions, ExportTransactionsParams};
//! use chrono::NaiveDate;
//!
//! # fn main() {
//! // Export all accounts
//! let accounts = export_accounts::call();
//! for account in accounts {
//!     println!("{}: {} {}",
//!         account.name,
//!         account.balance.amount,
//!         account.balance.currency
//!     );
//! }
//!
//! // Export transactions from a specific date
//! let params = ExportTransactionsParams {
//!     from_date: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date"),
//!     ..Default::default()
//! };
//! let response = export_transactions::call(params);
//! # }
//! ```
//!
//! ## Available Operations
//!
//! ### Accounts
//! - [`export_accounts::call()`] - Export all accounts with balances and metadata
//!
//! ### Categories
//! - [`export_categories::call()`] - Export all categories with budgets
//!
//! ### Transactions
//! - [`export_transactions::call()`] - Export transactions with flexible filtering
//!
//! ### Transfers (Experimental)
//! - `create_bank_transfer::call()` - Create bank transfers (requires `experimental` feature)
//!
//! ## Feature Flags
//!
//! - `experimental` - Enables experimental APIs like `create_bank_transfer` that may change
//!
//! ## MoneyMoney API Documentation
//!
//! For details on the underlying AppleScript API, see:
//! <https://moneymoney-app.com/api/>

use serde;
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
    #[cfg(feature = "experimental")]
    CreateBankTransfer(methods::create_bank_transfer::CreateBankTransferParams),
}

impl MoneymoneyActions {
    fn method_name(&self) -> String {
        match self {
            MoneymoneyActions::ExportAccounts => "exportAccounts".to_string(),
            MoneymoneyActions::ExportCategories => "exportCategories".to_string(),
            MoneymoneyActions::ExportTransactions(_) => "exportTransactions".to_string(),
            #[cfg(feature = "experimental")]
            MoneymoneyActions::CreateBankTransfer(_) => "createBankTransfer".to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("OSA script execution failed: {0}")]
    OsaScript(#[from] osascript::Error),

    #[error("Plist deserialization failed: {0}")]
    Plist(#[from] plist::Error),

    #[error("Received empty plist response from MoneyMoney")]
    EmptyPlist,

    #[error("Invalid currency code: {0}")]
    InvalidCurrency(String),

    #[error("Missing required parameter: {0}")]
    MissingRequiredParameter(&'static str),
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

pub fn call_action_plist<T>(action: MoneymoneyActions) -> Result<T, Error>
where
    T: DeserializeOwned + Serialize,
{
    let plist_response = call_action(action).map_err(|e| Error::OsaScript(e))?;

    match plist_response {
        Some(v) => Ok(plist::from_bytes(v.as_bytes()).map_err(|e| Error::Plist(e))?),
        None => Err(Error::EmptyPlist),
    }
}

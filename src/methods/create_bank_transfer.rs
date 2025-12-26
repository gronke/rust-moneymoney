//! Create bank transfers in MoneyMoney (experimental).
//!
//! This module provides functionality to create SEPA bank transfers programmatically.
//! This is an experimental feature and may change in future versions.
//!
//! # Feature Flag
//!
//! This module is only available when the `experimental` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! moneymoney = { version = "0.2", features = ["experimental"] }
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "experimental")]
//! # {
//! use moneymoney::create_bank_transfer::{self, CreateBankTransferParams};
//!
//! # fn main() -> Result<(), moneymoney::Error> {
//! let params = CreateBankTransferParams {
//!     from_account: Some("My Checking Account".to_string()),
//!     to: Some("John Doe".to_string()),
//!     iban: Some("DE89370400440532013000".to_string()),
//!     amount: Some(100.50),
//!     purpose: Some("Invoice payment".to_string()),
//!     ..Default::default()
//! };
//! let result = moneymoney::create_bank_transfer(params)?;
//! # Ok(())
//! # }
//! # }
//! ```

use crate::{call_action_plist, MoneymoneyActions};
use serde::{Deserialize, Serialize};

/// Parameters for creating a SEPA bank transfer.
///
/// All fields are optional, but typically you'll want to specify at least
/// `from_account`, `to`, `iban`, `amount`, and `purpose`.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "experimental")]
/// # {
/// use moneymoney::create_bank_transfer::CreateBankTransferParams;
///
/// let params = CreateBankTransferParams {
///     from_account: Some("My Checking".to_string()),
///     to: Some("Jane Doe".to_string()),
///     iban: Some("DE89370400440532013000".to_string()),
///     amount: Some(250.0),
///     purpose: Some("Rent payment".to_string()),
///     instrument_code: Some("TRF".to_string()), // Normal transfer
///     ..Default::default()
/// };
/// # }
/// ```
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateBankTransferParams {
    /// Source account (UUID, IBAN, account number, or account name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account: Option<String>,

    /// Recipient name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,

    /// Recipient IBAN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iban: Option<String>,

    /// Recipient BIC (Bank Identifier Code).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bic: Option<String>,

    /// Transfer amount in Euro.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,

    /// Purpose text for the transfer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,

    /// SEPA end-to-end reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endtoend_reference: Option<String>,

    /// SEPA purpose code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose_code: Option<String>,

    /// SEPA local instrument code.
    ///
    /// Use "TRF" for normal payments (default) and "INST" for instant payments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_code: Option<String>,

    /// Scheduled execution date in YYYY-MM-DD format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_date: Option<String>,

    /// Destination for the transfer.
    ///
    /// By default, a payment window will be opened in MoneyMoney.
    /// Set to "outbox" to silently save the payment to the outbox instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub into: Option<String>,
}

/// Create a bank transfer in MoneyMoney.
///
/// Creates a SEPA bank transfer with the specified parameters. By default, this opens
/// the MoneyMoney payment window for user confirmation. Use `into: Some("outbox".to_string())`
/// to save the transfer to the outbox without user interaction.
///
/// # Arguments
///
/// * `params` - Transfer parameters including account, recipient, amount, and SEPA details
///
/// # Returns
///
/// Returns a `Result` containing plist values representing the created transfer.
///
/// # Errors
///
/// Returns [`enum@crate::Error`] if:
/// - MoneyMoney is not running
/// - The OSA script execution fails
/// - Required parameters are missing or invalid
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "experimental")]
/// # {
/// use moneymoney::create_bank_transfer::{self, CreateBankTransferParams};
///
/// # fn main() -> Result<(), moneymoney::Error> {
/// let params = CreateBankTransferParams {
///     from_account: Some("My Checking".to_string()),
///     to: Some("Jane Doe".to_string()),
///     iban: Some("DE89370400440532013000".to_string()),
///     amount: Some(100.0),
///     purpose: Some("Payment".to_string()),
///     into: Some("outbox".to_string()), // Save to outbox without confirmation
///     ..Default::default()
/// };
/// let result = moneymoney::create_bank_transfer(params)?;
/// # Ok(())
/// # }
/// # }
/// ```
pub fn create_bank_transfer(
    params: CreateBankTransferParams,
) -> Result<Vec<plist::Value>, crate::Error> {
    call_action_plist(MoneymoneyActions::CreateBankTransfer(params))
}

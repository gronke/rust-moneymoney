//! Create SEPA direct debit orders in MoneyMoney (experimental).
//!
//! This module provides functionality to create SEPA direct debit orders programmatically.
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
//! use moneymoney::create_direct_debit::{self, CreateDirectDebitParams};
//!
//! # fn main() -> Result<(), moneymoney::Error> {
//! let params = CreateDirectDebitParams {
//!     from_account: Some("My Checking Account".to_string()),
//!     for_debtor: Some("John Doe".to_string()),
//!     iban: Some("DE89370400440532013000".to_string()),
//!     amount: Some(100.50),
//!     purpose: Some("Monthly membership fee".to_string()),
//!     mandate_reference: Some("MANDATE-12345".to_string()),
//!     mandate_date: Some("2024-01-15".to_string()),
//!     ..Default::default()
//! };
//! let result = create_direct_debit::call(params)?;
//! # Ok(())
//! # }
//! # }
//! ```

use crate::{call_action_plist, MoneymoneyActions};
use serde::{Deserialize, Serialize};

/// Parameters for creating a SEPA direct debit order.
///
/// All fields are optional, but typically you'll want to specify at least
/// `from_account`, `for_debtor`, `iban`, `amount`, `purpose`, `mandate_reference`,
/// and `mandate_date`.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "experimental")]
/// # {
/// use moneymoney::create_direct_debit::CreateDirectDebitParams;
///
/// let params = CreateDirectDebitParams {
///     from_account: Some("My Checking".to_string()),
///     for_debtor: Some("Customer Name".to_string()),
///     iban: Some("DE89370400440532013000".to_string()),
///     amount: Some(99.99),
///     purpose: Some("Invoice #12345".to_string()),
///     mandate_reference: Some("MREF-001".to_string()),
///     mandate_date: Some("2024-01-01".to_string()),
///     instrument_code: Some("CORE".to_string()),  // Core direct debit
///     sequence_code: Some("RCUR".to_string()),    // Recurring
///     ..Default::default()
/// };
/// # }
/// ```
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateDirectDebitParams {
    /// Source account (UUID, IBAN, account number, or account name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account: Option<String>,

    /// Debtor name (the person/entity being debited).
    #[serde(rename = "for", skip_serializing_if = "Option::is_none")]
    pub for_debtor: Option<String>,

    /// Debtor IBAN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iban: Option<String>,

    /// Debtor BIC (Bank Identifier Code).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bic: Option<String>,

    /// Direct debit amount in Euro.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,

    /// Purpose text for the direct debit.
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
    /// Use "CORE" for core direct debits (default) and "B2B" for business-to-business direct debits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_code: Option<String>,

    /// SEPA sequence code.
    ///
    /// - "RCUR": First and recurring direct debits (default)
    /// - "FNAL": Final direct debit
    /// - "OOFF": One-off direct debit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence_code: Option<String>,

    /// Mandate reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mandate_reference: Option<String>,

    /// Mandate date in YYYY-MM-DD format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mandate_date: Option<String>,

    /// Scheduled execution date in YYYY-MM-DD format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_date: Option<String>,

    /// Destination for the direct debit order.
    ///
    /// By default, a payment window will be opened in MoneyMoney.
    /// Set to "outbox" to silently save the direct debit to the outbox instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub into: Option<String>,
}

/// Create a SEPA direct debit order in MoneyMoney.
///
/// Creates a SEPA direct debit with the specified parameters. By default, this opens
/// the MoneyMoney payment window for user confirmation. Use `into: Some("outbox".to_string())`
/// to save the direct debit to the outbox without user interaction.
///
/// # Arguments
///
/// * `params` - Direct debit parameters including account, debtor, amount, mandate details, and SEPA codes
///
/// # Returns
///
/// Returns a `Result` containing plist values representing the created direct debit.
///
/// # Errors
///
/// Returns [`enum@crate::Error`] if:
/// - MoneyMoney is not running
/// - The OSA script execution fails
/// - Required parameters are missing or invalid
/// - The account doesn't support direct debits
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "experimental")]
/// # {
/// use moneymoney::create_direct_debit::{self, CreateDirectDebitParams};
///
/// # fn main() -> Result<(), moneymoney::Error> {
/// let params = CreateDirectDebitParams {
///     from_account: Some("My Checking".to_string()),
///     for_debtor: Some("Customer Corp".to_string()),
///     iban: Some("DE89370400440532013000".to_string()),
///     amount: Some(250.00),
///     purpose: Some("Subscription fee".to_string()),
///     mandate_reference: Some("MREF-2024-001".to_string()),
///     mandate_date: Some("2024-01-01".to_string()),
///     sequence_code: Some("RCUR".to_string()),
///     into: Some("outbox".to_string()), // Save to outbox without confirmation
///     ..Default::default()
/// };
/// let result = create_direct_debit::call(params)?;
/// # Ok(())
/// # }
/// # }
/// ```
pub fn call(params: CreateDirectDebitParams) -> Result<Vec<plist::Value>, crate::Error> {
    call_action_plist(MoneymoneyActions::CreateDirectDebit(params))
}

//! Export transactions from MoneyMoney.
//!
//! This module provides functionality to retrieve and filter transaction history
//! from the MoneyMoney application.
//!
//! # Example
//!
//! ```rust,no_run
//! use moneymoney::export_transactions;
//! use moneymoney::export_transactions::ExportTransactionsParams;
//! use chrono::NaiveDate;
//!
//! # fn main() -> Result<(), moneymoney::Error> {
//! // Export transactions from a specific date range
//! let params = ExportTransactionsParams::new(
//!     NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date")
//! ).to_date(NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid date"));
//!
//! let response = export_transactions::call(params)?;
//! println!("Found {} transactions", response.transactions.len());
//! # Ok(())
//! # }
//! ```

use crate::{call_action_plist, Error, MoneymoneyActions};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Parameters for filtering exported transactions.
///
/// Use the builder pattern to construct filtering criteria. The `from_date` is required,
/// while other parameters are optional.
///
/// # Example
///
/// ```rust
/// use moneymoney::export_transactions::ExportTransactionsParams;
/// use chrono::NaiveDate;
///
/// let params = ExportTransactionsParams::new(
///     NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date")
/// )
/// .to_date(NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid date"))
/// .from_account("DE89370400440532013000");
/// ```
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportTransactionsParams {
    /// Start date for transaction filtering (inclusive, required).
    pub from_date: NaiveDate,
    /// End date for transaction filtering (inclusive, optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_date: Option<NaiveDate>,
    /// Filter by account UUID or IBAN (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account: Option<String>,
    /// Filter by category name (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_category: Option<String>,
}

impl ExportTransactionsParams {
    /// Create a new ExportTransactionsParams with a required start date.
    ///
    /// # Arguments
    ///
    /// * `from_date` - The start date for transaction filtering (inclusive)
    pub fn new(from_date: NaiveDate) -> Self {
        Self {
            from_date,
            to_date: None,
            from_account: None,
            from_category: None,
        }
    }

    /// Set the end date for transaction filtering (inclusive).
    pub fn to_date(mut self, to_date: NaiveDate) -> Self {
        self.to_date = Some(to_date);
        self
    }

    /// Filter transactions by account UUID or IBAN.
    pub fn from_account(mut self, account: impl Into<String>) -> Self {
        self.from_account = Some(account.into());
        self
    }

    /// Filter transactions by category.
    pub fn from_category(mut self, category: impl Into<String>) -> Self {
        self.from_category = Some(category.into());
        self
    }
}

/// A single transaction record from MoneyMoney.
///
/// Contains all transaction details including dates, amount, parties, and categorization.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MoneymoneyTransaction {
    /// Unique transaction identifier.
    pub id: u64,
    /// Date the transaction was booked.
    pub booking_date: DateTime<Utc>,
    /// Date the transaction value was applied.
    pub value_date: DateTime<Utc>,
    /// Name of the transaction party (payee/payer).
    pub name: String,
    /// Transaction purpose/description text.
    pub purpose: Option<String>,
    /// Transaction amount (negative for expenses, positive for income).
    pub amount: f64,
    /// Currency code of the transaction.
    pub currency: String,
    /// UUID of the account this transaction belongs to.
    pub account_uuid: Uuid,
    /// Whether the transaction has been booked (confirmed by bank).
    pub booked: bool,
    /// UUID of the category this transaction is assigned to.
    pub category_uuid: Uuid,
    /// Whether the transaction has been manually checked/verified.
    pub checkmark: bool,
    /// Comment/note attached to the transaction.
    #[serde(default)]
    pub comment: String,
}

/// Response from the export transactions operation.
///
/// Contains metadata about the export and the list of transactions.
#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionsResponse {
    /// Creator/exporter information.
    pub creator: String,
    /// List of transactions matching the filter criteria.
    pub transactions: Vec<MoneymoneyTransaction>,
}

/// Export transactions from MoneyMoney with filtering.
///
/// Retrieves transaction history based on the provided filter parameters.
///
/// # Arguments
///
/// * `params` - Filter parameters constructed via [`ExportTransactionsParams`]
///
/// # Returns
///
/// Returns a `Result` containing a [`TransactionsResponse`] with matching transactions.
///
/// # Errors
///
/// Returns [`enum@Error`] if:
/// - MoneyMoney is not running
/// - The OSA script execution fails
/// - The response cannot be parsed
///
/// # Example
///
/// ```rust,no_run
/// use moneymoney::export_transactions;
/// use moneymoney::export_transactions::ExportTransactionsParams;
/// use chrono::NaiveDate;
///
/// # fn main() -> Result<(), moneymoney::Error> {
/// let params = ExportTransactionsParams::new(
///     NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date")
/// );
/// let response = export_transactions::call(params)?;
/// println!("Found {} transactions", response.transactions.len());
/// # Ok(())
/// # }
/// ```
pub fn call(params: ExportTransactionsParams) -> Result<TransactionsResponse, Error> {
    call_action_plist(MoneymoneyActions::ExportTransactions(params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    // Integration test - requires MoneyMoney running
    #[test]
    #[ignore]
    fn test_export_transactions() {
        let transaction_params =
            ExportTransactionsParams::new(NaiveDate::from_ymd_opt(2024, 1, 1).expect("Valid date"));
        let response = super::call(transaction_params);
        assert!(response.is_ok())
    }

    // Unit tests for ExportTransactionsParams builder pattern
    #[test]
    fn test_params_builder_basic() {
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let params = ExportTransactionsParams::new(from);

        assert_eq!(params.from_date, from);
        assert!(params.to_date.is_none());
        assert!(params.from_account.is_none());
        assert!(params.from_category.is_none());
    }

    #[test]
    fn test_params_builder_with_to_date() {
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let params = ExportTransactionsParams::new(from).to_date(to);

        assert_eq!(params.from_date, from);
        assert_eq!(params.to_date, Some(to));
    }

    #[test]
    fn test_params_builder_with_account() {
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let params = ExportTransactionsParams::new(from).from_account("DE89370400440532013000");

        assert_eq!(params.from_account, Some("DE89370400440532013000".to_string()));
    }

    #[test]
    fn test_params_builder_with_category() {
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let params = ExportTransactionsParams::new(from).from_category("Groceries");

        assert_eq!(params.from_category, Some("Groceries".to_string()));
    }

    #[test]
    fn test_params_builder_chaining() {
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let params = ExportTransactionsParams::new(from)
            .to_date(to)
            .from_account("test-account")
            .from_category("test-category");

        assert_eq!(params.from_date, from);
        assert_eq!(params.to_date, Some(to));
        assert_eq!(params.from_account, Some("test-account".to_string()));
        assert_eq!(params.from_category, Some("test-category".to_string()));
    }

    #[test]
    fn test_params_serialization() {
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let params = ExportTransactionsParams::new(from);

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"fromDate\":\"2024-01-01\""));
        assert!(!json.contains("toDate")); // None values skipped
    }

    #[test]
    fn test_params_serialization_with_optionals() {
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let params = ExportTransactionsParams::new(from)
            .to_date(to)
            .from_account("test");

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"fromDate\":\"2024-01-01\""));
        assert!(json.contains("\"toDate\":\"2024-12-31\""));
        assert!(json.contains("\"fromAccount\":\"test\""));
    }
}

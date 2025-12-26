//! Add transactions to offline accounts in MoneyMoney.
//!
//! This module provides functionality to programmatically add transactions to
//! offline accounts (manual accounts). This is useful for importing data from
//! external sources or creating test data.
//!
//! **Note**: This only works with offline/manual accounts, not online banking accounts.
//!
//! # Example
//!
//! ```rust,no_run
//! use moneymoney::add_transaction::{self, AddTransactionParams};
//! use chrono::NaiveDate;
//!
//! # fn main() -> Result<(), moneymoney::Error> {
//! let params = AddTransactionParams::new(
//!     "My Cash Account",
//!     NaiveDate::from_ymd_opt(2024, 12, 25).unwrap(),
//!     "Coffee Shop",
//!     -4.50
//! )
//! .purpose("Morning coffee")
//! .category("Food & Drinks");
//!
//! add_transaction::call(params)?;
//! # Ok(())
//! # }
//! ```

use crate::{call_action_void, MoneymoneyActions};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Parameters for adding a transaction to an offline account.
///
/// This is used to programmatically add transactions to manual/offline accounts.
/// All transactions must specify the account, date, payee/recipient, and amount.
///
/// # Example
///
/// ```rust
/// use moneymoney::add_transaction::AddTransactionParams;
/// use chrono::NaiveDate;
///
/// // Basic transaction
/// let params = AddTransactionParams::new(
///     "Cash",
///     NaiveDate::from_ymd_opt(2024, 12, 25).unwrap(),
///     "Grocery Store",
///     -50.0
/// );
///
/// // With optional fields
/// let params = AddTransactionParams::new(
///     "Cash",
///     NaiveDate::from_ymd_opt(2024, 12, 25).unwrap(),
///     "Grocery Store",
///     -50.0
/// )
/// .purpose("Weekly shopping")
/// .category("Groceries");
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddTransactionParams {
    /// Target offline account (UUID, account name, IBAN, or account number).
    pub to_account: String,

    /// Booking date in YYYY-MM-DD format.
    #[serde(serialize_with = "serialize_date")]
    pub on_date: NaiveDate,

    /// Debtor name (for incoming) or creditor name (for outgoing).
    pub to: String,

    /// Transaction amount in the account's currency.
    /// Positive for income, negative for expenses.
    pub amount: f64,

    /// Optional purpose/description text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,

    /// Optional category assignment (UUID or category name).
    /// Nested categories can be separated with backslashes.
    /// If not specified, auto-categorization will be applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

fn serialize_date<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&date.format("%Y-%m-%d").to_string())
}

impl AddTransactionParams {
    /// Create a new transaction to be added to an offline account.
    ///
    /// # Arguments
    ///
    /// * `to_account` - Target account identifier (UUID, name, IBAN, or account number)
    /// * `on_date` - Booking date for the transaction
    /// * `to` - Payee/recipient name (or debtor for income)
    /// * `amount` - Transaction amount (positive for income, negative for expenses)
    ///
    /// # Example
    ///
    /// ```rust
    /// use moneymoney::add_transaction::AddTransactionParams;
    /// use chrono::NaiveDate;
    ///
    /// let params = AddTransactionParams::new(
    ///     "My Cash Account",
    ///     NaiveDate::from_ymd_opt(2024, 12, 25).unwrap(),
    ///     "ATM Withdrawal",
    ///     100.0
    /// );
    /// ```
    pub fn new<S: Into<String>>(to_account: S, on_date: NaiveDate, to: S, amount: f64) -> Self {
        Self {
            to_account: to_account.into(),
            on_date,
            to: to.into(),
            amount,
            purpose: None,
            category: None,
        }
    }

    /// Set the purpose/description text for the transaction.
    ///
    /// # Arguments
    ///
    /// * `purpose` - Description or purpose of the transaction
    pub fn purpose<S: Into<String>>(mut self, purpose: S) -> Self {
        self.purpose = Some(purpose.into());
        self
    }

    /// Set the category for the transaction.
    ///
    /// # Arguments
    ///
    /// * `category` - Category identifier (UUID or name)
    ///
    /// # Note
    ///
    /// Nested categories can be separated with backslashes (e.g., "Food\\Restaurants").
    /// If not specified, MoneyMoney's auto-categorization will be applied.
    pub fn category<S: Into<String>>(mut self, category: S) -> Self {
        self.category = Some(category.into());
        self
    }
}

/// Add a transaction to an offline account in MoneyMoney.
///
/// Creates a new transaction in a manual/offline account. This is useful for:
/// - Importing transactions from external sources
/// - Adding cash transactions manually
/// - Creating test data for development
///
/// **Important**: This only works with offline accounts, not online banking accounts.
///
/// # Arguments
///
/// * `params` - Transaction details including account, date, payee, and amount
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns [`enum@crate::Error`] if:
/// - MoneyMoney is not running
/// - The specified account is not an offline account
/// - The OSA script execution fails
/// - Required parameters are invalid
///
/// # Example
///
/// ```rust,no_run
/// use moneymoney::add_transaction::{self, AddTransactionParams};
/// use chrono::NaiveDate;
///
/// # fn main() -> Result<(), moneymoney::Error> {
/// let params = AddTransactionParams::new(
///     "Cash Account",
///     NaiveDate::from_ymd_opt(2024, 12, 25).unwrap(),
///     "Restaurant",
///     -45.50
/// )
/// .purpose("Dinner with friends")
/// .category("Food & Drinks\\Restaurants");
///
/// add_transaction::call(params)?;
/// println!("Transaction added successfully!");
/// # Ok(())
/// # }
/// ```
pub fn call(params: AddTransactionParams) -> Result<(), crate::Error> {
    call_action_void(MoneymoneyActions::AddTransaction(params)).map_err(crate::Error::OsaScript)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_new() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        let params = AddTransactionParams::new("Cash", date, "Store", -10.0);

        assert_eq!(params.to_account, "Cash");
        assert_eq!(params.on_date, date);
        assert_eq!(params.to, "Store");
        assert_eq!(params.amount, -10.0);
        assert!(params.purpose.is_none());
        assert!(params.category.is_none());
    }

    #[test]
    fn test_params_builder_purpose() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        let params =
            AddTransactionParams::new("Cash", date, "Store", -10.0).purpose("Test purchase");

        assert_eq!(params.purpose, Some("Test purchase".to_string()));
    }

    #[test]
    fn test_params_builder_category() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        let params =
            AddTransactionParams::new("Cash", date, "Store", -10.0).category("Shopping\\Groceries");

        assert_eq!(params.category, Some("Shopping\\Groceries".to_string()));
    }

    #[test]
    fn test_params_builder_chaining() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        let params = AddTransactionParams::new("Cash", date, "Store", -10.0)
            .purpose("Weekly shopping")
            .category("Groceries");

        assert_eq!(params.purpose, Some("Weekly shopping".to_string()));
        assert_eq!(params.category, Some("Groceries".to_string()));
    }

    #[test]
    fn test_params_serialization() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        let params = AddTransactionParams::new("Cash", date, "Store", -10.0)
            .purpose("Test")
            .category("Food");

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("toAccount"));
        assert!(json.contains("onDate"));
        assert!(json.contains("2024-12-25"));
        assert!(json.contains("\"to\""));
        assert!(json.contains("amount"));
        assert!(json.contains("purpose"));
        assert!(json.contains("category"));
    }

    #[test]
    fn test_date_serialization() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let params = AddTransactionParams::new("Cash", date, "Store", -10.0);
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("2024-01-05"));
    }

    #[test]
    fn test_positive_amount_income() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        let params = AddTransactionParams::new("Cash", date, "Employer", 1000.0);
        assert_eq!(params.amount, 1000.0);
    }

    #[test]
    fn test_negative_amount_expense() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        let params = AddTransactionParams::new("Cash", date, "Store", -50.0);
        assert_eq!(params.amount, -50.0);
    }
}

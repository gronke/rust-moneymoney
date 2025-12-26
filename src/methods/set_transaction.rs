//! Modify existing transactions in MoneyMoney.
//!
//! This module provides functionality to update transaction properties such as:
//! - Checkmarks (marking transactions as reviewed)
//! - Category assignments
//! - Comments/notes
//!
//! Transaction IDs must be obtained first via `export_transactions` with `as: "plist"`.
//!
//! # Example
//!
//! ```rust,no_run
//! use moneymoney::set_transaction::{self, SetTransactionParams};
//!
//! # fn main() -> Result<(), moneymoney::Error> {
//! // Mark transaction as reviewed
//! let params = SetTransactionParams::new(12345)
//!     .checkmark("on");
//! set_transaction::call(params)?;
//!
//! // Assign category
//! let params = SetTransactionParams::new(12345)
//!     .category("Food & Drinks");
//! set_transaction::call(params)?;
//!
//! // Add comment
//! let params = SetTransactionParams::new(12345)
//!     .comment("Business expense - reimbursable");
//! set_transaction::call(params)?;
//! # Ok(())
//! # }
//! ```

use crate::{call_action, MoneymoneyActions};
use serde::{Deserialize, Serialize};

/// Parameters for modifying an existing transaction.
///
/// At least one of checkmark, category, or comment must be specified.
/// Transaction IDs must be obtained via `export_transactions` with `as: "plist"`.
///
/// # Example
///
/// ```rust
/// use moneymoney::set_transaction::SetTransactionParams;
///
/// // Mark as reviewed and categorize
/// let params = SetTransactionParams::new(12345)
///     .checkmark("on")
///     .category("Groceries")
///     .comment("Weekly shopping");
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetTransactionParams {
    /// Transaction ID (must be looked up with export_transactions).
    pub id: u64,

    /// Toggle checkmark to "on" or "off".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkmark_to: Option<String>,

    /// Category assignment (UUID or category name).
    /// Nested categories can be separated with backslashes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_to: Option<String>,

    /// Comment/note to add to the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_to: Option<String>,
}

impl SetTransactionParams {
    /// Create parameters for modifying a transaction.
    ///
    /// # Arguments
    ///
    /// * `id` - Transaction ID obtained from `export_transactions` as plist
    ///
    /// # Example
    ///
    /// ```rust
    /// use moneymoney::set_transaction::SetTransactionParams;
    ///
    /// let params = SetTransactionParams::new(12345);
    /// ```
    pub fn new(id: u64) -> Self {
        Self {
            id,
            checkmark_to: None,
            category_to: None,
            comment_to: None,
        }
    }

    /// Set the checkmark status.
    ///
    /// # Arguments
    ///
    /// * `value` - Either "on" or "off"
    ///
    /// # Example
    ///
    /// ```rust
    /// use moneymoney::set_transaction::SetTransactionParams;
    ///
    /// let params = SetTransactionParams::new(12345)
    ///     .checkmark("on");
    /// ```
    pub fn checkmark<S: Into<String>>(mut self, value: S) -> Self {
        self.checkmark_to = Some(value.into());
        self
    }

    /// Assign a category to the transaction.
    ///
    /// # Arguments
    ///
    /// * `category` - Category identifier (UUID or name)
    ///
    /// # Note
    ///
    /// Nested categories can be separated with backslashes (e.g., "Food\\Restaurants").
    ///
    /// # Example
    ///
    /// ```rust
    /// use moneymoney::set_transaction::SetTransactionParams;
    ///
    /// let params = SetTransactionParams::new(12345)
    ///     .category("Food & Drinks\\Restaurants");
    /// ```
    pub fn category<S: Into<String>>(mut self, category: S) -> Self {
        self.category_to = Some(category.into());
        self
    }

    /// Set a comment/note on the transaction.
    ///
    /// # Arguments
    ///
    /// * `comment` - Comment text to add
    ///
    /// # Example
    ///
    /// ```rust
    /// use moneymoney::set_transaction::SetTransactionParams;
    ///
    /// let params = SetTransactionParams::new(12345)
    ///     .comment("Business expense - needs reimbursement");
    /// ```
    pub fn comment<S: Into<String>>(mut self, comment: S) -> Self {
        self.comment_to = Some(comment.into());
        self
    }
}

/// Modify an existing transaction in MoneyMoney.
///
/// Updates transaction properties such as checkmark status, category assignment,
/// or comments. This is useful for:
/// - Bulk categorization of transactions
/// - Marking transactions as reviewed
/// - Adding notes or annotations programmatically
///
/// # Arguments
///
/// * `params` - Transaction modification parameters including the transaction ID
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns [`enum@crate::Error`] if:
/// - MoneyMoney is not running
/// - The transaction ID is invalid or not found
/// - The OSA script execution fails
///
/// # Example
///
/// ```rust,no_run
/// use moneymoney::{export_transactions, set_transaction};
/// use moneymoney::export_transactions::ExportTransactionsParams;
/// use moneymoney::set_transaction::SetTransactionParams;
/// use chrono::NaiveDate;
///
/// # fn main() -> Result<(), moneymoney::Error> {
/// // First, get transaction IDs
/// let from_date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
/// let params = ExportTransactionsParams::new(from_date);
/// let response = export_transactions::call(params)?;
///
/// // Then modify transactions
/// for transaction in &response.transactions {
///     if transaction.name.contains("Grocery") {
///         let params = SetTransactionParams::new(transaction.id)
///             .category("Groceries")
///             .checkmark("on");
///         set_transaction::call(params)?;
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn call(params: SetTransactionParams) -> Result<(), crate::Error> {
    call_action(MoneymoneyActions::SetTransaction(params)).map_err(crate::Error::OsaScript)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_new() {
        let params = SetTransactionParams::new(12345);
        assert_eq!(params.id, 12345);
        assert!(params.checkmark_to.is_none());
        assert!(params.category_to.is_none());
        assert!(params.comment_to.is_none());
    }

    #[test]
    fn test_params_checkmark() {
        let params = SetTransactionParams::new(12345).checkmark("on");
        assert_eq!(params.checkmark_to, Some("on".to_string()));
        assert!(params.category_to.is_none());
        assert!(params.comment_to.is_none());
    }

    #[test]
    fn test_params_category() {
        let params = SetTransactionParams::new(12345).category("Food");
        assert!(params.checkmark_to.is_none());
        assert_eq!(params.category_to, Some("Food".to_string()));
        assert!(params.comment_to.is_none());
    }

    #[test]
    fn test_params_comment() {
        let params = SetTransactionParams::new(12345).comment("Test note");
        assert!(params.checkmark_to.is_none());
        assert!(params.category_to.is_none());
        assert_eq!(params.comment_to, Some("Test note".to_string()));
    }

    #[test]
    fn test_params_builder_chaining() {
        let params = SetTransactionParams::new(12345)
            .checkmark("on")
            .category("Groceries")
            .comment("Weekly shopping");

        assert_eq!(params.id, 12345);
        assert_eq!(params.checkmark_to, Some("on".to_string()));
        assert_eq!(params.category_to, Some("Groceries".to_string()));
        assert_eq!(params.comment_to, Some("Weekly shopping".to_string()));
    }

    #[test]
    fn test_params_serialization() {
        let params = SetTransactionParams::new(12345)
            .checkmark("on")
            .category("Food")
            .comment("Note");

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("12345"));
        assert!(json.contains("checkmarkTo"));
        assert!(json.contains("categoryTo"));
        assert!(json.contains("commentTo"));
    }

    #[test]
    fn test_nested_category() {
        let params = SetTransactionParams::new(12345).category("Food\\Restaurants");
        assert_eq!(params.category_to, Some("Food\\Restaurants".to_string()));
    }

    #[test]
    fn test_checkmark_off() {
        let params = SetTransactionParams::new(12345).checkmark("off");
        assert_eq!(params.checkmark_to, Some("off".to_string()));
    }

    #[test]
    fn test_multiple_operations() {
        // Test that we can set all three fields
        let params = SetTransactionParams::new(999)
            .checkmark("on")
            .category("Test\\Category")
            .comment("Test comment with special chars: €$£");

        assert_eq!(params.id, 999);
        assert!(params.checkmark_to.is_some());
        assert!(params.category_to.is_some());
        assert!(params.comment_to.is_some());
    }
}

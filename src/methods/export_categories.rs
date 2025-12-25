//! Export categories and budgets from MoneyMoney.
//!
//! This module provides functionality to retrieve all categories with their
//! associated budgets from the MoneyMoney application.
//!
//! # Example
//!
//! ```rust,no_run
//! use moneymoney::export_categories;
//!
//! # fn main() -> Result<(), moneymoney::Error> {
//! let categories = export_categories::call()?;
//! for category in categories {
//!     if let Some(budget) = category.budget {
//!         println!("{}: {} {} budget, {} available",
//!             category.name,
//!             budget.amount,
//!             category.currency,
//!             budget.available
//!         );
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::{call_action_plist, Error, MoneymoneyActions};
use iso_currency::Currency;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Budget period type.
///
/// Represents the time period over which a budget is calculated.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase", untagged)]
pub enum Period {
    /// Quarterly budget period.
    Quaterly,
    /// Yearly budget period.
    Yearly,
    /// Total/lifetime budget.
    Total,
    /// Monthly budget period.
    Monthly,
}

/// Budget information for a category.
///
/// Contains the budgeted amount, available remaining amount, and the budget period.
#[derive(Serialize, Debug)]
pub struct MoneymoneyCategoryBudget {
    /// Total budgeted amount.
    pub amount: f64,
    /// Remaining available amount in the budget.
    pub available: f64,
    /// Budget period (monthly, yearly, etc.).
    pub period: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MaybeBudget {
    Full {
        amount: f64,
        available: f64,
        period: String,
    },
    Empty {},
}

fn untagged_to_option<'de, D>(deserializer: D) -> Result<Option<MoneymoneyCategoryBudget>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    match MaybeBudget::deserialize(deserializer) {
        Ok(MaybeBudget::Full {
            amount,
            available,
            period,
        }) => Ok(Some(MoneymoneyCategoryBudget {
            amount,
            available,
            period,
        })),
        Ok(MaybeBudget::Empty {}) => Ok(None),
        Err(_) => Ok(None),
    }
}

/// A MoneyMoney category with optional budget information.
///
/// Categories are used to organize and classify transactions. Each category
/// can have an associated budget that tracks spending limits.
#[derive(Serialize, Deserialize, Debug)]
pub struct MoneymoneyCategory {
    /// Unique category identifier.
    pub uuid: Uuid,
    /// Category display name.
    pub name: String,
    /// Optional budget information for this category.
    #[serde(deserialize_with = "untagged_to_option")]
    pub budget: Option<MoneymoneyCategoryBudget>,
    /// Category currency.
    pub currency: Currency,
    /// Whether this is a default system category.
    pub default: bool,
    /// Whether this is a category group.
    pub group: bool,
    /// Category icon as binary data.
    pub icon: plist::Data,
    /// Display indentation level.
    pub indentation: u8,
}

/// Export all categories from MoneyMoney.
///
/// Retrieves all categories including category groups and their associated budgets.
///
/// # Returns
///
/// Returns a `Result` containing a vector of [`MoneymoneyCategory`] on success.
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
/// use moneymoney::export_categories;
///
/// # fn main() -> Result<(), moneymoney::Error> {
/// let categories = export_categories::call()?;
/// for category in categories.iter().filter(|c| !c.group) {
///     if let Some(budget) = &category.budget {
///         println!("{}: Budget {} {}, Available {}",
///             category.name,
///             budget.amount,
///             category.currency,
///             budget.available
///         );
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn call() -> Result<Vec<MoneymoneyCategory>, Error> {
    call_action_plist(MoneymoneyActions::ExportCategories)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration test - requires MoneyMoney running
    #[test]
    #[ignore]
    fn test_list_categories() {
        assert!(super::call().is_ok())
    }

    // Unit tests for MoneymoneyCategoryBudget and budget deserialization
    #[test]
    fn test_budget_deserialization_full() {
        let json = r#"{
            "uuid": "12345678-1234-1234-1234-123456789012",
            "name": "Groceries",
            "budget": {
                "amount": 500.0,
                "available": 250.0,
                "period": "monthly"
            },
            "currency": "EUR",
            "default": false,
            "group": false,
            "icon": "",
            "indentation": 0
        }"#;

        let category: MoneymoneyCategory = serde_json::from_str(json).unwrap();
        assert!(category.budget.is_some());
        let budget = category.budget.unwrap();
        assert_eq!(budget.amount, 500.0);
        assert_eq!(budget.available, 250.0);
        assert_eq!(budget.period, "monthly");
    }

    #[test]
    fn test_budget_deserialization_empty() {
        let json = r#"{
            "uuid": "12345678-1234-1234-1234-123456789012",
            "name": "No Budget Category",
            "budget": {},
            "currency": "EUR",
            "default": false,
            "group": false,
            "icon": "",
            "indentation": 0
        }"#;

        let category: MoneymoneyCategory = serde_json::from_str(json).unwrap();
        assert!(category.budget.is_none());
    }

    #[test]
    fn test_category_deserialization_without_budget() {
        let json = r#"{
            "uuid": "12345678-1234-1234-1234-123456789012",
            "name": "Test Category",
            "budget": {},
            "currency": "USD",
            "default": true,
            "group": false,
            "icon": "",
            "indentation": 1
        }"#;

        let category: MoneymoneyCategory = serde_json::from_str(json).unwrap();
        assert_eq!(category.name, "Test Category");
        assert_eq!(category.currency.code(), "USD");
        assert!(category.default);
        assert!(!category.group);
        assert_eq!(category.indentation, 1);
        assert!(category.budget.is_none());
    }

    #[test]
    fn test_category_group() {
        let json = r#"{
            "uuid": "12345678-1234-1234-1234-123456789012",
            "name": "Category Group",
            "budget": {},
            "currency": "EUR",
            "default": false,
            "group": true,
            "icon": "",
            "indentation": 0
        }"#;

        let category: MoneymoneyCategory = serde_json::from_str(json).unwrap();
        assert!(category.group);
        assert!(category.budget.is_none());
    }

    #[test]
    fn test_budget_serialization() {
        let budget = MoneymoneyCategoryBudget {
            amount: 1000.0,
            available: 750.0,
            period: "yearly".to_string(),
        };

        let json = serde_json::to_string(&budget).unwrap();
        assert!(json.contains("\"amount\":1000.0"));
        assert!(json.contains("\"available\":750.0"));
        assert!(json.contains("\"period\":\"yearly\""));
    }

    #[test]
    fn test_category_with_various_periods() {
        for period in &["monthly", "yearly", "quarterly", "total"] {
            let json = format!(
                r#"{{
                    "uuid": "12345678-1234-1234-1234-123456789012",
                    "name": "Test",
                    "budget": {{
                        "amount": 100.0,
                        "available": 50.0,
                        "period": "{}"
                    }},
                    "currency": "EUR",
                    "default": false,
                    "group": false,
                    "icon": "",
                    "indentation": 0
                }}"#,
                period
            );

            let category: MoneymoneyCategory = serde_json::from_str(&json).unwrap();
            assert!(category.budget.is_some());
            assert_eq!(category.budget.unwrap().period, *period);
        }
    }

    #[test]
    fn test_category_with_various_currencies() {
        for currency_code in &["EUR", "USD", "GBP", "JPY"] {
            let json = format!(
                r#"{{
                    "uuid": "12345678-1234-1234-1234-123456789012",
                    "name": "Test",
                    "budget": {{}},
                    "currency": "{}",
                    "default": false,
                    "group": false,
                    "icon": "",
                    "indentation": 0
                }}"#,
                currency_code
            );

            let category: MoneymoneyCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(category.currency.code(), *currency_code);
        }
    }
}

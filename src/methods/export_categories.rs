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

    #[test]
    fn test_list_categories() {
        assert!(super::call().is_ok())
    }
}

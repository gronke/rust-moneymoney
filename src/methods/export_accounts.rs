//! Export accounts from MoneyMoney.
//!
//! This module provides functionality to retrieve all accounts with their balances
//! and metadata from the MoneyMoney application.
//!
//! # Example
//!
//! ```rust,no_run
//! # fn main() -> Result<(), moneymoney::Error> {
//! let accounts = moneymoney::export_accounts()?;
//! for account in accounts.iter().filter(|a| !a.group) {
//!     println!("{}: {} {}",
//!         account.name,
//!         account.balance.amount,
//!         account.balance.currency
//!     );
//! }
//! # Ok(())
//! # }
//! ```

use crate::{call_action_plist, MoneymoneyActions};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

/// The type of a MoneyMoney account.
///
/// This enum represents the various account types supported by MoneyMoney.
/// It handles both English and German localized strings during deserialization.
///
/// # Serialization
///
/// When serializing, English strings are used (e.g., "Cash", "Giro account").
///
/// # Deserialization
///
/// Both English and German strings are supported (e.g., "Cash"/"Bargeld",
/// "Giro account"/"Girokonto"). Unknown account type strings are captured
/// as [`MoneymoneyAccountType::Custom`].
#[derive(Debug)]
pub enum MoneymoneyAccountType {
    /// Account group (container for organizing other accounts).
    Group,
    /// Checking/Giro account.
    Giro,
    /// Savings account.
    Savings,
    /// Fixed term deposit account.
    FixedTermDeposit,
    /// Loan account.
    Loan,
    /// Credit card account.
    CreditCard,
    /// Cash account.
    Cash,
    /// Other account type.
    Other,
    /// Custom account type with a user-defined string.
    Custom(String),
}

impl Serialize for MoneymoneyAccountType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            MoneymoneyAccountType::Group => "Account group", // Kontengruppe
            MoneymoneyAccountType::Giro => "Giro account",
            MoneymoneyAccountType::Savings => "Savings account",
            MoneymoneyAccountType::FixedTermDeposit => "Fixed term deposit",
            MoneymoneyAccountType::Loan => "Loan account",
            MoneymoneyAccountType::CreditCard => "Credit card",
            MoneymoneyAccountType::Cash => "Cash", // Bargeld (matches AccountTypeCash)
            MoneymoneyAccountType::Other => "Other",
            MoneymoneyAccountType::Custom(value) => value,
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for MoneymoneyAccountType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Account group" | "Kontengruppe" => Ok(MoneymoneyAccountType::Group),
            "Giro account" | "Girokonto" => Ok(MoneymoneyAccountType::Giro),
            "Savings account" | "Sparkonto" => Ok(MoneymoneyAccountType::Savings),
            "Fixed term deposit" | "Festgeldanlage" => Ok(MoneymoneyAccountType::FixedTermDeposit),
            "Loan account" | "Darlehenskonto" => Ok(MoneymoneyAccountType::Loan),
            "Credit card" | "Kreditkarte" => Ok(MoneymoneyAccountType::CreditCard),
            "Cash" | "Bargeld" => Ok(MoneymoneyAccountType::Cash),
            "Other" | "Sonstige" => Ok(MoneymoneyAccountType::Other),
            other => Ok(MoneymoneyAccountType::Custom(other.to_string())),
        }
    }
}

/// The balance of an account with amount and currency.
///
/// # Fields
///
/// * `amount` - The account balance as a floating-point number
/// * `currency` - The ISO 4217 currency code (e.g., EUR, USD)
///
/// # Errors
///
/// Deserialization fails if the currency code is invalid, returning [`crate::Error::InvalidCurrency`].
#[derive(Serialize, Deserialize, Debug)]
#[serde(try_from = "Vec<BalanceTuple>")]
pub struct AccountBalance {
    /// The balance amount.
    pub amount: f64,
    /// The currency of the balance.
    pub currency: iso_currency::Currency,
}

#[derive(Debug, Deserialize)]
struct BalanceTuple(f64, String);

impl TryFrom<Vec<BalanceTuple>> for AccountBalance {
    type Error = crate::Error;

    fn try_from(tuple: Vec<BalanceTuple>) -> Result<Self, Self::Error> {
        let balance = tuple.first().ok_or(crate::Error::EmptyPlist)?;

        let currency = iso_currency::Currency::from_code(&balance.1)
            .ok_or_else(|| crate::Error::InvalidCurrency(balance.1.clone()))?;

        Ok(AccountBalance {
            amount: balance.0,
            currency,
        })
    }
}

/// A MoneyMoney account with all its metadata.
///
/// This struct represents a complete account record from MoneyMoney, including
/// balance, type, timestamps, and organizational information.
///
/// # Fields
///
/// * `account_number` - The account number
/// * `attributes` - Custom attributes dictionary
/// * `balance` - Current balance with currency
/// * `bank_code` - Bank identification code (BLZ/BIC)
/// * `currency` - Account currency code
/// * `group` - Whether this is an account group
/// * `icon` - Account icon as binary data
/// * `indentation` - Indentation level for display hierarchy
/// * `name` - Account display name
/// * `owner` - Account owner name
/// * `portfolio` - Whether this is a portfolio/investment account
/// * `refresh_timestamp` - Last synchronization timestamp
/// * `type` - The account type (giro, savings, credit card, etc.)
/// * `uuid` - Unique identifier for the account
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MoneymoneyAccount {
    /// The account number.
    pub account_number: String,
    /// Custom account attributes.
    pub attributes: plist::Dictionary,
    /// Current account balance with currency.
    pub balance: AccountBalance,
    /// Bank identification code.
    pub bank_code: String,
    /// Account currency code.
    pub currency: String,
    /// Whether this is an account group.
    pub group: bool,
    /// Account icon as binary data.
    pub icon: plist::Data,
    /// Display indentation level.
    pub indentation: u8,
    /// Account display name.
    pub name: String,
    /// Account owner name.
    pub owner: String,
    /// Whether this is a portfolio account.
    pub portfolio: bool,
    /// Last refresh timestamp.
    pub refresh_timestamp: DateTime<Utc>,
    /// Account type.
    pub r#type: MoneymoneyAccountType,
    /// Unique account identifier.
    pub uuid: Uuid,
}

/// Export all accounts from MoneyMoney.
///
/// Retrieves all accounts including account groups and their current balances.
///
/// # Returns
///
/// Returns a `Result` containing a vector of [`MoneymoneyAccount`] on success.
///
/// # Errors
///
/// Returns [`enum@crate::Error`] if:
/// - MoneyMoney is not running
/// - The OSA script execution fails
/// - The response cannot be parsed
/// - Invalid currency codes are encountered
///
/// # Example
///
/// ```rust,no_run
/// # fn main() -> Result<(), moneymoney::Error> {
/// let accounts = moneymoney::export_accounts()?;
/// for account in accounts.iter().filter(|a| !a.group) {
///     println!("{}: {} {}",
///         account.name,
///         account.balance.amount,
///         account.balance.currency
///     );
/// }
/// # Ok(())
/// # }
/// ```
pub fn export_accounts() -> Result<Vec<MoneymoneyAccount>, crate::Error> {
    call_action_plist(MoneymoneyActions::ExportAccounts)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration test - requires MoneyMoney running
    #[test]
    #[ignore]
    fn test_list_accounts() {
        let accounts = super::export_accounts().expect("Failed to retrieve accounts");
        assert!(!accounts.is_empty());
        assert!(
            accounts
                .iter()
                .any(|account| account.name == "All accounts"),
            "Expected at least one account with name 'All accounts', found none!"
        );
    }

    // Unit tests for MoneymoneyAccountType serialization
    #[test]
    fn test_account_type_deserialize_english() {
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Account group""#).unwrap(),
            MoneymoneyAccountType::Group
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Giro account""#).unwrap(),
            MoneymoneyAccountType::Giro
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Savings account""#).unwrap(),
            MoneymoneyAccountType::Savings
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Fixed term deposit""#).unwrap(),
            MoneymoneyAccountType::FixedTermDeposit
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Loan account""#).unwrap(),
            MoneymoneyAccountType::Loan
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Credit card""#).unwrap(),
            MoneymoneyAccountType::CreditCard
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Cash""#).unwrap(),
            MoneymoneyAccountType::Cash
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Other""#).unwrap(),
            MoneymoneyAccountType::Other
        ));
    }

    #[test]
    fn test_account_type_deserialize_german() {
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Kontengruppe""#).unwrap(),
            MoneymoneyAccountType::Group
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Girokonto""#).unwrap(),
            MoneymoneyAccountType::Giro
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Sparkonto""#).unwrap(),
            MoneymoneyAccountType::Savings
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Festgeldanlage""#).unwrap(),
            MoneymoneyAccountType::FixedTermDeposit
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Darlehenskonto""#).unwrap(),
            MoneymoneyAccountType::Loan
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Kreditkarte""#).unwrap(),
            MoneymoneyAccountType::CreditCard
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Bargeld""#).unwrap(),
            MoneymoneyAccountType::Cash
        ));
        assert!(matches!(
            serde_json::from_str::<MoneymoneyAccountType>(r#""Sonstige""#).unwrap(),
            MoneymoneyAccountType::Other
        ));
    }

    #[test]
    fn test_account_type_deserialize_custom() {
        match serde_json::from_str::<MoneymoneyAccountType>(r#""Investment Account""#).unwrap() {
            MoneymoneyAccountType::Custom(s) => assert_eq!(s, "Investment Account"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_account_type_serialize() {
        assert_eq!(serde_json::to_string(&MoneymoneyAccountType::Cash).unwrap(), r#""Cash""#);
        assert_eq!(
            serde_json::to_string(&MoneymoneyAccountType::Giro).unwrap(),
            r#""Giro account""#
        );
        assert_eq!(
            serde_json::to_string(&MoneymoneyAccountType::Custom("Test".to_string())).unwrap(),
            r#""Test""#
        );
    }

    // Unit tests for AccountBalance TryFrom
    #[test]
    fn test_account_balance_try_from_valid() {
        let tuple = vec![BalanceTuple(100.50, "EUR".to_string())];
        let balance = AccountBalance::try_from(tuple).unwrap();
        assert_eq!(balance.amount, 100.50);
        assert_eq!(balance.currency.code(), "EUR");
    }

    #[test]
    fn test_account_balance_try_from_invalid_currency() {
        let tuple = vec![BalanceTuple(100.50, "INVALID".to_string())];
        let result = AccountBalance::try_from(tuple);
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::Error::InvalidCurrency(code) => assert_eq!(code, "INVALID"),
            _ => panic!("Expected InvalidCurrency error"),
        }
    }

    #[test]
    fn test_account_balance_try_from_empty() {
        let tuple: Vec<BalanceTuple> = vec![];
        let result = AccountBalance::try_from(tuple);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::Error::EmptyPlist));
    }

    #[test]
    fn test_account_balance_try_from_various_currencies() {
        for code in &["USD", "GBP", "JPY", "CHF"] {
            let tuple = vec![BalanceTuple(123.45, code.to_string())];
            let balance = AccountBalance::try_from(tuple).unwrap();
            assert_eq!(balance.currency.code(), *code);
        }
    }
}

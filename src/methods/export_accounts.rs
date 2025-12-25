use crate::{call_action_plist, MoneymoneyActions};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug)]
pub enum MoneymoneyAccountType {
    Group,
    Giro,
    Savings,
    FixedTermDeposit,
    Loan,
    CreditCard,
    Cash,
    Other,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(try_from = "Vec<BalanceTuple>")]
pub struct AccountBalance {
    pub amount: f64,
    pub currency: iso_currency::Currency,
}

#[derive(Debug, Deserialize)]
struct BalanceTuple(f64, String);

impl TryFrom<Vec<BalanceTuple>> for AccountBalance {
    type Error = crate::Error;

    fn try_from(tuple: Vec<BalanceTuple>) -> Result<Self, Self::Error> {
        let balance = tuple
            .get(0)
            .ok_or(crate::Error::EmptyPlist)?;

        let currency = iso_currency::Currency::from_code(&balance.1)
            .ok_or_else(|| crate::Error::InvalidCurrency(balance.1.clone()))?;

        Ok(AccountBalance {
            amount: balance.0,
            currency,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MoneymoneyAccount {
    pub account_number: String,
    pub attributes: plist::Dictionary,
    pub balance: AccountBalance,
    pub bank_code: String,
    pub currency: String,
    pub group: bool,
    pub icon: plist::Data,
    pub indentation: u8,
    pub name: String,
    pub owner: String,
    pub portfolio: bool,
    pub refresh_timestamp: DateTime<Utc>,
    pub r#type: MoneymoneyAccountType,
    pub uuid: Uuid,
}

pub fn call() -> Result<Vec<MoneymoneyAccount>, crate::Error> {
    call_action_plist(MoneymoneyActions::ExportAccounts)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_list_accounts() {
        let accounts = super::call().expect("Failed to retrieve accounts");
        assert!(accounts.len() > 0);
        assert!(
            accounts
                .iter()
                .any(|account| account.name == "All accounts"),
            "Expected at least one account with name 'All accounts', found none!"
        );
    }
}

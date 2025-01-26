use crate::{MoneymoneyActions, call_action_plist};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
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
            MoneymoneyAccountType::Cash => todo!(),
            MoneymoneyAccountType::Other => "Other",
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
            other => Err(de::Error::unknown_variant(other, &[
                "Account group",
                "Giro account",
                "Savings account",
                "Fixed term deposit",
                "Loan account",
                "Credit card",
                "Cash",
                "Other",
            ])),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MoneymoneyAccount {
    pub account_number: String,
    pub attributes: plist::Dictionary,
    pub balance: Vec<(f64, String)>,
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

pub fn call() -> Vec<MoneymoneyAccount> {
    call_action_plist(MoneymoneyActions::ExportAccounts)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_list_accounts() {
        let accounts = super::call();
        assert!(accounts.len() > 0);
        let account = &accounts[0];
        assert_eq!(account.name, "All accounts");
    }
}

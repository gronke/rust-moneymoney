use crate::{MoneymoneyActions, call_action_plist};
use iso_currency::Currency;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase", untagged)]
pub enum Period {
    Quaterly,
    Yearly,
    Total,
    Monthly,
}

#[derive(Serialize, Debug)]
pub struct MoneymoneyCategoryBudget {
    pub amount: f64,
    pub available: f64,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct MoneymoneyCategory {
    pub uuid: Uuid,
    pub name: String,
    #[serde(deserialize_with = "untagged_to_option")]
    pub budget: Option<MoneymoneyCategoryBudget>,
    pub currency: Currency,
    pub default: bool,
    pub group: bool,
    pub icon: plist::Data,
    pub indentation: u8,
}

pub fn call() -> Vec<MoneymoneyCategory> {
    call_action_plist(MoneymoneyActions::ExportCategories)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_list_categories() {
        super::call();
        // ToDo: build testing automation for proper testing beyond a functional check
    }
}

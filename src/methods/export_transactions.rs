use crate::{call_action_plist, Error, MoneymoneyActions};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn fail_start_date() -> NaiveDate {
    panic!("Start date required");
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportTransactionsParams {
    #[serde(default = "fail_start_date")]
    pub from_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_category: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MoneymoneyTransaction {
    pub id: u64,
    pub booking_date: DateTime<Utc>,
    pub value_date: DateTime<Utc>,
    pub name: String,
    pub purpose: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub account_uuid: Uuid,
    pub booked: bool,
    pub category_uuid: Uuid,
    pub checkmark: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionsResponse {
    pub creator: String,
    pub transactions: Vec<MoneymoneyTransaction>,
}

pub fn call(params: ExportTransactionsParams) -> Result<TransactionsResponse, Error> {
    call_action_plist(MoneymoneyActions::ExportTransactions(params.into()))
}

#[cfg(test)]
mod tests {

    use super::ExportTransactionsParams;

    #[test]
    fn test_export_transactions() {
        let transaction_params = ExportTransactionsParams {
            from_date: chrono::NaiveDate::from_ymd_opt(2024, 01, 01).unwrap(),
            ..Default::default()
        };
        let response = super::call(transaction_params);
        assert!(response.is_ok())
    }
}

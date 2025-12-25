use crate::{call_action_plist, Error, MoneymoneyActions};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportTransactionsParams {
    pub from_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account: Option<String>,
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
        let transaction_params = ExportTransactionsParams::new(
            chrono::NaiveDate::from_ymd_opt(2024, 01, 01).expect("Valid date")
        );
        let response = super::call(transaction_params);
        assert!(response.is_ok())
    }
}

use crate::{call_action_plist, MoneymoneyActions};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateBankTransferParams {
    // UUID, IBAN, account number or account name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account: Option<String>,
    // Recipient name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    // Recipient IBAN
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iban: Option<String>,
    // Recipient BIC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bic: Option<String>,
    // Amount in Euro
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    // Purpose text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    // SEPA end-to-end reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endtoend_reference: Option<String>,
    // SEPA purpose code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose_code: Option<String>,
    // SEPA local instrument code. Use TRF for normal payments and INST for instant payments. Default is TRF
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_code: Option<String>,
    // Scheduled date (YYYY-MM-DD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_date: Option<String>,
    // By default a payment window will be opened. If this parameter is set to "outbox", the payment will be silently saved into the outbox instead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub into: Option<String>,
}

pub fn call(params: CreateBankTransferParams) -> Vec<plist::Value> {
    call_action_plist(MoneymoneyActions::CreateBankTransfer(params.into()))
}

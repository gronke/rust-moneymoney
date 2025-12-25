use serde;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

mod methods;
pub use methods::*;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum MoneymoneyActions {
    ExportAccounts,
    ExportCategories,
    ExportTransactions(methods::export_transactions::ExportTransactionsParams),
    #[cfg(feature = "experimental")]
    CreateBankTransfer(methods::create_bank_transfer::CreateBankTransferParams),
}

impl MoneymoneyActions {
    fn method_name(&self) -> String {
        match self {
            MoneymoneyActions::ExportAccounts => "exportAccounts".to_string(),
            MoneymoneyActions::ExportCategories => "exportCategories".to_string(),
            MoneymoneyActions::ExportTransactions(_) => "exportTransactions".to_string(),
            #[cfg(feature = "experimental")]
            MoneymoneyActions::CreateBankTransfer(_) => "createBankTransfer".to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("OSA script execution failed: {0}")]
    OsaScript(#[from] osascript::Error),

    #[error("Plist deserialization failed: {0}")]
    Plist(#[from] plist::Error),

    #[error("Received empty plist response from MoneyMoney")]
    EmptyPlist,

    #[error("Invalid currency code: {0}")]
    InvalidCurrency(String),

    #[error("Missing required parameter: {0}")]
    MissingRequiredParameter(&'static str),
}

#[derive(Serialize, Deserialize)]
struct ScriptAction {
    method: String,
    args: MoneymoneyActions,
}

pub fn call_action(action: MoneymoneyActions) -> Result<Option<String>, osascript::Error> {
    let params = ScriptAction {
        method: action.method_name(),
        args: action,
    };
    let script = osascript::JavaScript::new(
        "
        if ($params.args) {
            $params.args['as'] = 'plist';
        }
        return Application('MoneyMoney')[$params.method]($params.args || []);
    ",
    );
    script.execute_with_params(&params)
}

pub fn call_action_plist<T>(action: MoneymoneyActions) -> Result<T, Error>
where
    T: DeserializeOwned + Serialize,
{
    let plist_response = call_action(action).map_err(|e| Error::OsaScript(e))?;

    match plist_response {
        Some(v) => Ok(plist::from_bytes(v.as_bytes()).map_err(|e| Error::Plist(e))?),
        None => Err(Error::EmptyPlist),
    }
}

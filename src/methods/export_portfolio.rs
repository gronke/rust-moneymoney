//! Export securities portfolio holdings from MoneyMoney.
//!
//! This module provides functionality to export portfolio/securities data including
//! holdings, asset allocations, and investment positions.
//!
//! # Example
//!
//! ```rust,no_run
//! use moneymoney::export_portfolio::{self, ExportPortfolioParams};
//!
//! # fn main() -> Result<(), moneymoney::Error> {
//! // Export entire portfolio
//! let portfolio = export_portfolio::call(ExportPortfolioParams::default())?;
//! for holding in &portfolio.securities {
//!     println!("{}: {} shares", holding.name, holding.quantity);
//! }
//!
//! // Export portfolio for a specific account
//! let params = ExportPortfolioParams::default()
//!     .from_account("Investment Account");
//! let portfolio = export_portfolio::call(params)?;
//! # Ok(())
//! # }
//! ```

use crate::{call_action_plist, MoneymoneyActions};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Parameters for exporting portfolio data.
///
/// All fields are optional. Without filters, exports the entire portfolio
/// across all accounts.
///
/// # Example
///
/// ```rust
/// use moneymoney::export_portfolio::ExportPortfolioParams;
///
/// // Export entire portfolio
/// let params = ExportPortfolioParams::default();
///
/// // Export specific account's portfolio
/// let params = ExportPortfolioParams::default()
///     .from_account("12345678-1234-1234-1234-123456789012");
///
/// // Export specific asset class
/// let params = ExportPortfolioParams::default()
///     .from_asset_class("Stocks");
/// ```
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportPortfolioParams {
    /// Filter by account (UUID, IBAN, account number, account name, or account group name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account: Option<String>,

    /// Filter by asset class (UUID or asset class name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_asset_class: Option<String>,
}

impl ExportPortfolioParams {
    /// Create a new `ExportPortfolioParams` with all filters disabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter portfolio by account.
    ///
    /// # Arguments
    ///
    /// * `account` - Account identifier (UUID, IBAN, account number, name, or group name)
    pub fn from_account<S: Into<String>>(mut self, account: S) -> Self {
        self.from_account = Some(account.into());
        self
    }

    /// Filter portfolio by asset class.
    ///
    /// # Arguments
    ///
    /// * `asset_class` - Asset class identifier (UUID or name)
    pub fn from_asset_class<S: Into<String>>(mut self, asset_class: S) -> Self {
        self.from_asset_class = Some(asset_class.into());
        self
    }
}

/// Response from the export portfolio operation.
///
/// Contains the securities holdings and related metadata.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportPortfolioResponse {
    /// List of securities holdings.
    #[serde(default)]
    pub securities: Vec<Security>,
}

/// Represents a security/investment holding in the portfolio.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Security {
    /// Security unique identifier.
    pub uuid: Uuid,

    /// Security name.
    pub name: String,

    /// ISIN (International Securities Identification Number).
    #[serde(default)]
    pub isin: String,

    /// WKN (Wertpapierkennnummer - German securities identification code).
    #[serde(default)]
    pub wkn: String,

    /// Security symbol/ticker.
    #[serde(default)]
    pub symbol: String,

    /// Quantity of shares/units held.
    pub quantity: f64,

    /// Account UUID where the security is held.
    pub account_uuid: Uuid,

    /// Account name where the security is held.
    pub account_name: String,

    /// Current market price per unit.
    #[serde(default)]
    pub market_price: f64,

    /// Currency of the market price.
    #[serde(default)]
    pub currency: String,

    /// Market value (quantity × market price).
    pub market_value: f64,

    /// Purchase price per unit (average cost basis).
    #[serde(default)]
    pub purchase_price: f64,

    /// Purchase value (quantity × purchase price).
    #[serde(default)]
    pub purchase_value: f64,

    /// Profit/loss amount (market value - purchase value).
    #[serde(default)]
    pub profit: f64,

    /// Profit/loss percentage.
    #[serde(default)]
    pub profit_percent: f64,

    /// Asset class name (e.g., "Stocks", "Bonds", "ETFs").
    #[serde(default)]
    pub asset_class: String,
}

/// Export portfolio holdings from MoneyMoney.
///
/// Exports securities/investment data including positions, values, and profit/loss.
/// The returned data includes all holdings across selected accounts with current
/// market values and performance metrics.
///
/// # Arguments
///
/// * `params` - Portfolio export parameters for filtering by account or asset class
///
/// # Returns
///
/// Returns a `Result` containing the portfolio data with all securities holdings.
///
/// # Errors
///
/// Returns [`enum@crate::Error`] if:
/// - MoneyMoney is not running
/// - The OSA script execution fails
/// - The response cannot be parsed
///
/// # Example
///
/// ```rust,no_run
/// use moneymoney::export_portfolio::{self, ExportPortfolioParams};
///
/// # fn main() -> Result<(), moneymoney::Error> {
/// // Export entire portfolio
/// let portfolio = export_portfolio::call(ExportPortfolioParams::default())?;
///
/// for holding in &portfolio.securities {
///     println!("{}: {} shares @ {} {} = {} {}",
///         holding.name,
///         holding.quantity,
///         holding.market_price,
///         holding.currency,
///         holding.market_value,
///         holding.currency
///     );
///     println!("  Profit/Loss: {:.2}%", holding.profit_percent);
/// }
/// # Ok(())
/// # }
/// ```
pub fn call(params: ExportPortfolioParams) -> Result<ExportPortfolioResponse, crate::Error> {
    call_action_plist(MoneymoneyActions::ExportPortfolio(params))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_default() {
        let params = ExportPortfolioParams::default();
        assert!(params.from_account.is_none());
        assert!(params.from_asset_class.is_none());
    }

    #[test]
    fn test_params_builder_from_account() {
        let params =
            ExportPortfolioParams::new().from_account("12345678-1234-1234-1234-123456789012");
        assert_eq!(params.from_account, Some("12345678-1234-1234-1234-123456789012".to_string()));
        assert!(params.from_asset_class.is_none());
    }

    #[test]
    fn test_params_builder_from_asset_class() {
        let params = ExportPortfolioParams::new().from_asset_class("Stocks");
        assert!(params.from_account.is_none());
        assert_eq!(params.from_asset_class, Some("Stocks".to_string()));
    }

    #[test]
    fn test_params_builder_chaining() {
        let params = ExportPortfolioParams::new()
            .from_account("Investment Account")
            .from_asset_class("ETFs");
        assert_eq!(params.from_account, Some("Investment Account".to_string()));
        assert_eq!(params.from_asset_class, Some("ETFs".to_string()));
    }

    #[test]
    fn test_params_serialization() {
        let params = ExportPortfolioParams::new()
            .from_account("test-account")
            .from_asset_class("Bonds");
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("fromAccount"));
        assert!(json.contains("fromAssetClass"));
    }

    #[test]
    fn test_security_deserialization() {
        let json = r#"{
            "uuid": "12345678-1234-1234-1234-123456789012",
            "name": "Apple Inc.",
            "isin": "US0378331005",
            "wkn": "865985",
            "symbol": "AAPL",
            "quantity": 10.0,
            "accountUuid": "87654321-4321-4321-4321-210987654321",
            "accountName": "Investment Account",
            "marketPrice": 150.0,
            "currency": "USD",
            "marketValue": 1500.0,
            "purchasePrice": 120.0,
            "purchaseValue": 1200.0,
            "profit": 300.0,
            "profitPercent": 25.0,
            "assetClass": "Stocks"
        }"#;

        let security: Security = serde_json::from_str(json).unwrap();
        assert_eq!(security.name, "Apple Inc.");
        assert_eq!(security.isin, "US0378331005");
        assert_eq!(security.quantity, 10.0);
        assert_eq!(security.market_value, 1500.0);
        assert_eq!(security.profit_percent, 25.0);
    }

    #[test]
    #[ignore]
    fn test_export_portfolio() {
        let params = ExportPortfolioParams::default();
        let result = call(params);
        assert!(result.is_ok());
    }
}

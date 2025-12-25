//! MoneyMoney API method implementations.
//!
//! Each submodule corresponds to a MoneyMoney AppleScript method:
//!
//! - [`export_accounts`] - Retrieve account information with balances
//! - [`export_categories`] - Retrieve category and budget information
//! - [`export_transactions`] - Retrieve and filter transaction history
//! - [`create_bank_transfer`] - Create bank transfers (experimental, requires `experimental` feature)
//!
//! All methods communicate with the MoneyMoney application via OSA (Open Scripting Architecture)
//! and return properly typed Rust structures.

pub mod export_accounts;
pub mod export_categories;
pub mod export_transactions;

#[cfg(feature = "experimental")]
pub mod create_bank_transfer;

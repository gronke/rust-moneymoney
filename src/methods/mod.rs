//! MoneyMoney API method implementations.
//!
//! Each submodule corresponds to a MoneyMoney AppleScript method:
//!
//! ## Export Methods
//! - [`export_accounts`] - Retrieve account information with balances
//! - [`export_categories`] - Retrieve category and budget information
//! - [`export_transactions`] - Retrieve and filter transaction history
//! - [`export_portfolio`] - Retrieve securities and portfolio holdings
//!
//! ## Transaction Management
//! - [`add_transaction`] - Add transactions to offline accounts
//! - [`set_transaction`] - Modify existing transaction properties
//!
//! ## Payment Operations (Experimental)
//! - [`create_bank_transfer`] - Create bank transfers (requires `experimental` feature)
//! - [`create_direct_debit`] - Create SEPA direct debit orders (requires `experimental` feature)
//!
//! All methods communicate with the MoneyMoney application via OSA (Open Scripting Architecture)
//! and return properly typed Rust structures.

pub mod add_transaction;
pub mod export_accounts;
pub mod export_categories;
pub mod export_portfolio;
pub mod export_transactions;
pub mod set_transaction;

#[cfg(feature = "experimental")]
pub mod create_bank_transfer;

#[cfg(feature = "experimental")]
pub mod create_direct_debit;

// Re-export functions at crate root for ergonomic API
pub use add_transaction::add_transaction;
pub use export_accounts::export_accounts;
pub use export_categories::export_categories;
pub use export_portfolio::export_portfolio;
pub use export_transactions::export_transactions;
pub use set_transaction::set_transaction;

#[cfg(feature = "experimental")]
pub use create_bank_transfer::create_bank_transfer;

#[cfg(feature = "experimental")]
pub use create_direct_debit::create_direct_debit;

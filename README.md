# MoneyMoney Rust Interface

[![crates.io][crates-io-img]](https://lib.rs/crates/moneymoney)
[![Documentation](https://docs.rs/moneymoney/badge.svg)](https://docs.rs/moneymoney)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A safe, ergonomic Rust interface to the [MoneyMoney](https://moneymoney-app.com/) macOS application via AppleScript.

## Features

- **Type-safe API**: All data structures use proper Rust types
- **Comprehensive error handling**: All operations return `Result` types
- **Serde integration**: Full serialization/deserialization support
- **Zero unsafe code**: Pure safe Rust implementation
- **Well documented**: Comprehensive API documentation with examples

## Requirements

- **macOS**: MoneyMoney is a macOS-only application
- **MoneyMoney app**: Must be installed and running
- **Rust**: 1.70 or later

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
moneymoney = "0.2"
```

## Quick Start

```rust
use moneymoney::{export_accounts, export_transactions, ExportTransactionsParams};
use chrono::NaiveDate;

fn main() -> Result<(), moneymoney::Error> {
    // Export all accounts
    let accounts = export_accounts::call()?;
    for account in accounts {
        println!("{}: {} {}",
            account.name,
            account.balance.amount,
            account.balance.currency
        );
    }

    // Export transactions from a specific date
    let params = ExportTransactionsParams::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date")
    );
    let response = export_transactions::call(params)?;
    println!("Found {} transactions", response.transactions.len());

    Ok(())
}
```

## API Coverage

MoneyMoney AppleScript API implementation status:

- [x] **Export Accounts** - Retrieve all accounts with balances
- [x] **Export Categories** - Retrieve categories and budgets
- [x] **Export Transactions** - Retrieve and filter transactions
- [x] **Create Bank Transfer** - Create transfers (experimental feature)
- [ ] Export Portfolio
- [ ] Create Batch Transfer
- [ ] Create Direct Debit
- [ ] Create Batch Direct Debit
- [ ] Add Transaction Set

## Usage Examples

### Export Accounts

```rust
use moneymoney::export_accounts;

let accounts = export_accounts::call()?;
for account in accounts.iter().filter(|a| !a.group) {
    println!("Account: {} - Balance: {} {}",
        account.name,
        account.balance.amount,
        account.balance.currency
    );
}
```

### Filter Transactions by Date Range

```rust
use moneymoney::{export_transactions, ExportTransactionsParams};
use chrono::NaiveDate;

let params = ExportTransactionsParams::new(
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
)
.to_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());

let response = export_transactions::call(params)?;
```

### Export Categories with Budgets

```rust
use moneymoney::export_categories;

let categories = export_categories::call()?;
for category in categories {
    if let Some(budget) = category.budget {
        println!("{}: Budget {} {}, Available {}",
            category.name,
            budget.amount,
            category.currency,
            budget.available
        );
    }
}
```

## Error Handling

All functions return `Result<T, Error>` for proper error handling:

```rust
use moneymoney::{export_accounts, Error};

match export_accounts::call() {
    Ok(accounts) => {
        println!("Retrieved {} accounts", accounts.len());
    }
    Err(Error::OsaScript(e)) => {
        eprintln!("MoneyMoney communication error: {}", e);
    }
    Err(Error::EmptyPlist) => {
        eprintln!("No data returned");
    }
    Err(e) => {
        eprintln!("Error: {:?}", e);
    }
}
```

## Feature Flags

- `experimental` - Enables experimental APIs that may change between versions

```toml
[dependencies]
moneymoney = { version = "0.2", features = ["experimental"] }
```

## Documentation

- **API Documentation**: [docs.rs/moneymoney](https://docs.rs/moneymoney)
- **MoneyMoney AppleScript API**: [moneymoney-app.com/api](https://moneymoney-app.com/api/)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development

```bash
# Run tests
cargo test

# Build documentation
cargo doc --open

# Run clippy
cargo clippy

# Format code
cargo fmt
```

## License

MIT License - See [LICENSE](LICENSE) file for details

## Author

Stefan Grönke - [stefan@gronke.net](mailto:stefan@gronke.net)

## Resources

- [MoneyMoney Application](https://moneymoney-app.com/)
- [MoneyMoney AppleScript Documentation](https://moneymoney-app.com/api/)
- [crates.io](https://crates.io/crates/moneymoney)
- [GitHub Repository](https://github.com/gronke/rust-moneymoney)

[crates-io-img]: https://img.shields.io/crates/v/moneymoney.svg

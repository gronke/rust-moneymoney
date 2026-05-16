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

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
moneymoney = "0.2"
```

## Quick Start

```rust
use moneymoney::export_transactions::ExportTransactionsParams;
use chrono::NaiveDate;

fn main() -> Result<(), moneymoney::Error> {
    // Export all accounts
    let accounts = moneymoney::export_accounts()?;
    for account in accounts {
        if let Some(balance) = &account.balance {
            println!("{}: {} {}", account.name, balance.amount, balance.currency);
        }
    }

    // Export transactions from a specific date
    let params = ExportTransactionsParams::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date")
    );
    let response = moneymoney::export_transactions(params)?;
    println!("Found {} transactions", response.transactions.len());

    Ok(())
}
```

## API Coverage

All 8 MoneyMoney AppleScript API methods are implemented:

- **Export Accounts** - Retrieve all accounts with balances
- **Export Categories** - Retrieve categories and budgets
- **Export Transactions** - Retrieve and filter transactions
- **Export Portfolio** - Retrieve securities and portfolio holdings
- **Add Transaction** - Add transactions to offline accounts
- **Set Transaction** - Modify existing transaction properties
- **Create Bank Transfer** - Create SEPA bank transfers (experimental)
- **Create Direct Debit** - Create SEPA direct debit orders (experimental)

## Usage Examples

### Export Accounts

```rust
let accounts = moneymoney::export_accounts()?;
for account in accounts.iter().filter(|a| !a.group) {
    if let Some(balance) = &account.balance {
        println!("Account: {} - Balance: {} {}",
            account.name,
            balance.amount,
            balance.currency
        );
    }
}
```

### Filter Transactions by Date Range

```rust
use moneymoney::export_transactions::ExportTransactionsParams;
use chrono::NaiveDate;

let params = ExportTransactionsParams::new(
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
)
.to_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());

let response = moneymoney::export_transactions(params)?;
```

### Export Categories with Budgets

```rust
let categories = moneymoney::export_categories()?;
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

All functions return `Result<T, Error>`:

```rust
use moneymoney::Error;

match moneymoney::export_accounts() {
    Ok(accounts) => println!("Retrieved {} accounts", accounts.len()),
    Err(Error::OsaScript(e)) => eprintln!("MoneyMoney error: {}", e),
    Err(e) => eprintln!("Error: {:?}", e),
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

Contributions are welcome — please open a Pull Request. See
[DEVELOPMENT.md](DEVELOPMENT.md) for the test layout (unit, integration,
schema-drift) and the `make` quality-check targets.

## License

MIT License - See [LICENSE](LICENSE) file for details

## Author

Stefan Grönke - [stefan@gronke.net](mailto:stefan@gronke.net)

## Automated Coding Assistance

This project is developed with automated coding assistance, particularly for test automation and quality assurance.
Needless to say, Cargo releases and tags are reviewed and signed on dedicated systems.

## Resources

- [MoneyMoney Application](https://moneymoney-app.com/)
- [MoneyMoney AppleScript Documentation](https://moneymoney-app.com/api/)
- [crates.io](https://crates.io/crates/moneymoney)
- [GitHub Repository](https://github.com/gronke/rust-moneymoney)

[crates-io-img]: https://img.shields.io/crates/v/moneymoney.svg

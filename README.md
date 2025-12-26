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

MoneyMoney AppleScript API implementation status (8/8 = 100%):

### Export Operations
- [x] **Export Accounts** - Retrieve all accounts with balances
- [x] **Export Categories** - Retrieve categories and budgets
- [x] **Export Transactions** - Retrieve and filter transactions
- [x] **Export Portfolio** - Retrieve securities and portfolio holdings

### Transaction Management
- [x] **Add Transaction** - Add transactions to offline accounts
- [x] **Set Transaction** - Modify existing transaction properties

### Payment Operations (Experimental)
- [x] **Create Bank Transfer** - Create SEPA bank transfers
- [x] **Create Direct Debit** - Create SEPA direct debit orders

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

## Testing

The library includes both unit tests and integration tests.

### Unit Tests

Unit tests don't require MoneyMoney to be running:

```bash
# Run all unit tests
cargo test --lib

# Run doc tests
cargo test --doc

# Run all checks (format, lint, test, doc)
make check
```

### Integration Tests

Integration tests interact with MoneyMoney and require setup.

#### Option 1: Test with Production Database (Recommended)

This is the simplest approach for quick testing:

1. **Keep MoneyMoney running** with your normal data
2. **Create test accounts** manually:
   - Account name: `test-cash` (Type: Cash, Currency: EUR)
   - Account name: `test-checking` (Type: Giro, Currency: EUR)
3. **Run roundtrip tests**:
   ```bash
   cargo test --test roundtrip_tests -- --ignored --nocapture
   ```
4. **Review results** in MoneyMoney - all test data is in `test-` prefixed accounts
5. **Clean up** by deleting test accounts when done

**Why this is safe:**
- Tests only modify accounts prefixed with `test-`
- Your real accounts are never touched
- You can review everything before cleanup

#### Option 2: Isolated Test Database

For complete isolation from production data:

1. **Quit MoneyMoney** if running
2. **Launch test instance**:
   ```bash
   ./scripts/run_test_moneymoney.sh
   ```
   This will:
   - Backup your production database
   - Launch MoneyMoney with isolated test database
   - Show instructions for next steps

3. **Create test accounts** in the test MoneyMoney instance:
   - `test-cash` (Cash Account, EUR)
   - `test-checking` (Giro Account, EUR)

4. **Run tests**:
   ```bash
   cargo test --test roundtrip_tests -- --ignored --nocapture
   ```

5. **Restore production**:
   ```bash
   ./scripts/restore_production_moneymoney.sh
   ```

#### What the Tests Do

The roundtrip integration tests validate the complete workflow:

1. ✅ Verify test accounts exist
2. ✅ Add 12 realistic transactions (German merchants: Media Markt, REWE, Bäckerei Schmidt, etc.)
3. ✅ Export and verify all transactions
4. ✅ Modify transactions (add checkmarks, comments, categories)
5. ✅ Verify persistence across multiple reads

**Note:** Tests intentionally do NOT clean up data so you can review results in MoneyMoney.

#### Running Specific Tests

```bash
# Run all roundtrip tests
cargo test --test roundtrip_tests -- --ignored --nocapture

# Run specific test
cargo test --test roundtrip_tests test_roundtrip_add_read_modify_transactions -- --ignored --nocapture

# Run standard integration tests (existing data export tests)
cargo test --test integration_tests -- --ignored --nocapture
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Commands

```bash
# Run all quality checks
make check

# Individual commands
make test      # Run unit and doc tests
make lint      # Run clippy
make fmt       # Format code
make doc       # Build documentation
make audit     # Security audit

# Format and check
make all
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

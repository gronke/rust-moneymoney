# moneymoney-cli

Command-line interface to the [MoneyMoney](https://moneymoney-app.com/)
macOS application. Built on the [`moneymoney`](https://crates.io/crates/moneymoney)
library — install this crate if you want to query MoneyMoney from the
terminal; depend on the library directly if you want to embed it in your
own Rust code.

## Requirements

- **macOS** — MoneyMoney is macOS-only.
- **MoneyMoney running and unlocked** when the CLI is invoked.

## Installation

```bash
cargo install moneymoney-cli
```

The installed binary is named `moneymoney`. To also enable experimental
subcommands (`create bank-transfer`):

```bash
cargo install moneymoney-cli --features experimental
```

## Usage

All output is pretty-printed JSON on stdout. Pipe to `jq` for filtering
and reshaping.

### `moneymoney export accounts`

```bash
moneymoney export accounts                          # real accounts only
moneymoney export accounts --include-group-accounts # also account groups/folders
moneymoney export accounts --include-icon-data      # include raw icon bytes
```

### `moneymoney export categories`

```bash
moneymoney export categories
moneymoney export categories --include-group-categories
moneymoney export categories --include-icon-data
```

### `moneymoney export transactions`

```bash
moneymoney export transactions --from-date 2024-01-01
moneymoney export transactions --from-date 2024-01-01 --to-date 2024-12-31
moneymoney export transactions --from-date 2024-06-01 --from-account <uuid-or-iban>
moneymoney export transactions --from-date 2024-06-01 --from-category Groceries
```

`--from-date` is required (ISO 8601 `YYYY-MM-DD`). `--to-date`,
`--from-account`, and `--from-category` are optional filters.

### `moneymoney create bank-transfer` (experimental)

Requires `cargo install moneymoney-cli --features experimental`. Reads
SEPA bank-transfer parameters as JSON from stdin (or a file path):

```bash
moneymoney create bank-transfer < params.json
moneymoney create bank-transfer params.json
echo '{ ... }' | moneymoney create bank-transfer -
```

## Output formats

Currently only JSON. Additional formats (CSV, table) may follow — see
[issue #17](https://github.com/gronke/rust-moneymoney/issues/17).

## Exit codes

| Code | Meaning |
|-----:|---------|
| 0    | Success |
| 1    | Runtime failure (e.g. MoneyMoney not running, OSA error) |
| 2    | Argument parse error (invalid date, unknown subcommand, missing required flag) |

## Repository

Source, issues, and contributing guide:
<https://github.com/gronke/rust-moneymoney>. Contributors should also
read [`DEVELOPMENT.md`](https://github.com/gronke/rust-moneymoney/blob/main/DEVELOPMENT.md).

## License

MIT — see [LICENSE](LICENSE).

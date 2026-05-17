# Development

Working on the `moneymoney` crate itself. End-user / library-consumer docs live in [README.md](README.md).

## Cargo features

| Feature | Default? | Effect |
|---|---|---|
| `cli` | yes | Pulls `clap` and `serde_json` as runtime deps; gates the `moneymoney` binary at `src/bin/moneymoney/main.rs` via `required-features = ["cli"]`. |
| `experimental` | no | Exposes WIP library APIs (`create_bank_transfer`, `create_direct_debit`) and the matching `create` subcommands in the CLI. |
| `test-utils` | no | Internal test scaffolding (`src/test_utils.rs`). |

Library consumers who don't want the CLI's runtime deps in their tree
should set `default-features = false` (see README). The `cli` feature
exists so that `cargo install moneymoney` works without flags while
still letting library consumers opt out.

## Quality checks

```bash
make check     # everything below

make test      # unit and doc tests
make lint      # clippy
make fmt       # rustfmt
make doc       # build documentation
make audit     # security audit
make all       # format and check
```

## Testing

### Unit tests

```bash
cargo test --lib
```

### Integration tests

Integration tests are `#[ignore]`-gated and require a running MoneyMoney
with seeded test accounts.

```bash
cargo test --test roundtrip_tests -- --ignored --nocapture --test-threads=1
```

Tests only touch `test-` prefixed accounts and never modify real data.

**Seeding accounts** (one-time): `scripts/create_test_accounts.sh` creates
the six offline accounts the tests expect, one per type the API can
deserialise:

- `test-cash` (Cash account)
- `test-giro` (Giro account)
- `test-savings` (Savings account)
- `test-fixed-term` (Fixed term deposit)
- `test-loan` (Loan account)
- `test-creditcard` (Credit card)

`scripts/delete_test_accounts.sh` removes them when you're done.

**Isolated database** (recommended if you don't want to risk touching
your production MoneyMoney data): `scripts/run_test_moneymoney.sh` backs
up your production container and launches MoneyMoney pointed at an
isolated sandbox under `.test_data/`. When done,
`scripts/restore_production_moneymoney.sh` swaps back.

### Schema-drift tests

Two safety nets that catch when the Rust binding falls out of sync with the
installed MoneyMoney app.

**Response schema** (`tests/transaction_plist_schema.rs`) — iterates every
`.plist` fixture under `tests/fixtures/transaction_exports/` and asserts each
one deserialises into `MoneymoneyTransaction` (which carries
`#[serde(deny_unknown_fields)]`) and that the union of observed keys matches
the expected schema. No running MoneyMoney needed.

```bash
cargo test --test transaction_plist_schema
```

To extend the corpus with a real-world capture, sanitise UUIDs and personal
data and drop the `.plist` into `tests/fixtures/transaction_exports/`. The
iterator picks it up automatically; no test code changes are needed unless
you're also introducing a new key — in which case extend
`MoneymoneyTransaction` and `EXPECTED_KEYS` in the same change.

**Command parameters** (`tests/system_integration/sdef.rs`) — reads
MoneyMoney's `MoneyMoney.sdef` at runtime and asserts each documented
parameter maps to a serde field on our parameter struct. The sdef path
resolves from:

1. `$MONEYMONEY_SDEF_PATH` if set, otherwise
2. `/Applications/MoneyMoney.app/Contents/Resources/MoneyMoney.sdef`.

If neither resolves, the test prints `SKIP:` and returns cleanly, so plain
`cargo test` stays green on machines without MoneyMoney installed.

```bash
# Picks up the installed app's sdef automatically on macOS:
cargo test --test system_integration

# Or point at a custom location (sandboxed installs, CI, etc.):
MONEYMONEY_SDEF_PATH=/path/to/MoneyMoney.sdef \
    cargo test --test system_integration
```

The sdef file is proprietary to the MoneyMoney app and stays gitignored —
never check it in.

### Live schema validation (optional)

Beyond the fixture corpus, `tests/transaction_plist_schema.rs` ships an
`#[ignore]`-gated `live_export_has_no_unknown_keys` that runs against the
actual MoneyMoney instance and asserts no unknown keys appear. Useful for
catching drift in production data the fixture corpus may not yet exercise.

```bash
cargo test --test transaction_plist_schema -- --ignored
```

# Development

Working on the `moneymoney` crate itself. End-user / library-consumer docs live in [README.md](README.md).

## Workspace layout

This repository is a Cargo workspace with two crates:

- **`moneymoney`** (root) — the library. MSRV `1.62`. No `clap` dependency.
- **`moneymoney-cli`** (`moneymoney-cli/`) — the CLI binary (`moneymoney`).
  MSRV `1.85`, depends on `moneymoney` via path + version.

The split keeps the library's dependency footprint small for downstream
consumers and lets the CLI use a newer toolchain without dragging the
library along.

## Quality checks

```bash
make check          # everything below

make test           # lib unit/doc tests + CLI tests
make lint           # clippy across the workspace
make fmt            # rustfmt across the workspace
make version-check  # lib ↔ CLI version alignment
make doc            # build documentation (library only)
make audit          # security audit
make all            # format and check
```

## Testing

### Unit tests

```bash
cargo test --lib
```

### Integration tests

Integration tests require MoneyMoney to be running with two test accounts:

1. Create offline accounts named `test-cash` (Cash) and `test-checking` (Giro) in EUR.
2. Run: `cargo test --test roundtrip_tests -- --ignored --nocapture --test-threads=1`
3. Clean up test accounts when done.

Tests only modify `test-` prefixed accounts and never touch real data.

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

## Releasing

The two crates are versioned **independently but manually aligned**: the
CLI tracks the library. Every library release implies a CLI release of
equal or greater bump; the CLI may also release on its own between
library releases for CLI-only changes (new subcommand, output format, …).

The CI job `Version Alignment` (`scripts/check_version_alignment.py`,
also run by `make version-check`) enforces two invariants on every PR
and push to `main`:

1. `moneymoney-cli`'s `[dependencies] moneymoney = { version = "..." }`
   must match the library's current major.minor.
2. `moneymoney-cli`'s package version must be ≥ the library's version.

So bumping the library without also bumping the CLI (and its dep pin)
fails CI immediately.

### Checklist

1. Decide the library bump (patch / minor / major) following SemVer.
2. Decide the CLI bump — must be **at least** the library's bump level.
   For library-only releases without CLI source changes, this is still a
   CLI patch bump so the published binary pins the fresh lib version.
3. Update three places in lockstep:
   - `Cargo.toml` → `[package] version`
   - `moneymoney-cli/Cargo.toml` → `[package] version`
   - `moneymoney-cli/Cargo.toml` → `[dependencies] moneymoney = { version = "X.Y" }`
4. Run `make check` locally — `version-check` must pass.
5. Update `CHANGELOG.md` (if present) for both crates.
6. Commit with a message naming both versions, e.g.
   `chore: release moneymoney 0.3.0 + moneymoney-cli 0.3.0`.
7. Tag both releases on the commit:
   `git tag moneymoney-v0.3.0 moneymoney-cli-v0.3.0`.
8. Publish **library first**, then **CLI** (the CLI's `version = "..."`
   pin on `moneymoney` resolves against crates.io at publish time):
   ```bash
   cargo publish -p moneymoney
   cargo publish -p moneymoney-cli
   ```
9. Push the commit and tags.

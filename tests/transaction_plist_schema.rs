//! Pin the `MoneymoneyTransaction` plist schema against captured fixtures.
//!
//! The sdef doesn't document response keys, so we keep a corpus of plist samples under
//! `tests/fixtures/transaction_exports/` covering different transaction shapes (cash,
//! SEPA, FX, batched direct debits, …). Two invariants fire together:
//!
//!   1. *Every* fixture must deserialize cleanly into `TransactionsResponse`. Because
//!      `MoneymoneyTransaction` is declared with `#[serde(deny_unknown_fields)]`, this
//!      fails the moment a fixture contains a key the struct doesn't know about — i.e.
//!      MoneyMoney added a new field and the binding hasn't caught up.
//!
//!   2. The *union* of keys observed across all fixtures must equal `EXPECTED_KEYS`.
//!      This catches the other direction: someone added a key to the struct (and to
//!      `EXPECTED_KEYS`) but never extended any fixture to exercise it — so the
//!      coverage claim is hollow.
//!
//! Adding a new fixture: drop a `.plist` file in
//! `tests/fixtures/transaction_exports/`. Real-world captures are encouraged — sanitise
//! UUIDs and personal data first. The directory is iterated automatically; no test
//! needs to be edited unless you're introducing a new key.
//!
//! Adding a new key to the binding:
//!   - extend `MoneymoneyTransaction` with the field,
//!   - extend `EXPECTED_KEYS` below,
//!   - extend at least one fixture so the key actually appears in the corpus.
//!
//! Failure to do all three is exactly the kind of drift this test is designed to
//! surface.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use moneymoney::export_transactions::TransactionsResponse;

/// Every camelCase key that `MoneymoneyTransaction` is expected to model. Extend in
/// lockstep with the struct definition.
const EXPECTED_KEYS: &[&str] = &[
    "id",
    "bookingDate",
    "valueDate",
    "name",
    "accountNumber",
    "bankCode",
    "transactionCode",
    "textKeyExtension",
    "purposeCode",
    "bookingKey",
    "primanotaNumber",
    "batchReference",
    "endToEndReference",
    "creditorId",
    "returnReason",
    "category",
    "purpose",
    "bookingText",
    "amount",
    "currency",
    "accountUuid",
    "booked",
    "categoryUuid",
    "checkmark",
    "mandateReference",
    "comment",
];

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/transaction_exports")
}

fn fixture_paths() -> Vec<PathBuf> {
    let dir = fixture_dir();
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {dir:?}: {e}"))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("plist"))
        .collect();
    paths.sort(); // deterministic iteration order so failures point at the same file
    assert!(!paths.is_empty(), "fixture corpus must not be empty");
    paths
}

fn keys_in(path: &PathBuf) -> HashSet<String> {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let root: plist::Value =
        plist::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {path:?}: {e}"));
    root.as_dictionary()
        .and_then(|d| d.get("transactions"))
        .and_then(|t| t.as_array())
        .into_iter()
        .flatten()
        .filter_map(|t| t.as_dictionary())
        .flat_map(|d| d.keys().cloned())
        .collect()
}

#[test]
fn every_fixture_deserialises() {
    for path in fixture_paths() {
        let bytes = fs::read(&path).unwrap();
        // deny_unknown_fields on MoneymoneyTransaction makes this fail loudly if the
        // fixture carries a key the struct doesn't model.
        let response: TransactionsResponse =
            plist::from_bytes(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert!(
            !response.transactions.is_empty(),
            "{} has no transactions; fixtures should exercise at least one",
            path.display()
        );
    }
}

#[test]
fn fixture_corpus_covers_every_expected_key() {
    let union: HashSet<String> = fixture_paths().iter().flat_map(keys_in).collect();
    let expected: HashSet<&str> = EXPECTED_KEYS.iter().copied().collect();
    let observed: HashSet<&str> = union.iter().map(String::as_str).collect();

    let unknown: Vec<&&str> = observed.difference(&expected).collect();
    assert!(
        unknown.is_empty(),
        "fixture corpus contains keys not in EXPECTED_KEYS: {unknown:?} — \
         either extend the binding or remove them from fixtures"
    );

    let uncovered: Vec<&&str> = expected.difference(&observed).collect();
    assert!(
        uncovered.is_empty(),
        "EXPECTED_KEYS references fields that no fixture exercises: {uncovered:?} — \
         add at least one fixture variant where each key is present"
    );
}

/// Live cross-check against a running MoneyMoney instance. Ignored by default because
/// it requires the user's local app; CI doesn't have MoneyMoney installed.
/// Run with:
///     cargo test --test transaction_plist_schema -- --ignored
///
/// This is the strongest signal: it catches schema drift in the actual data the
/// caller sees, not just in pinned fixtures.
#[test]
#[ignore]
fn live_export_has_no_unknown_keys() {
    use chrono::NaiveDate;
    use moneymoney::export_transactions::{export_transactions, ExportTransactionsParams};

    let params =
        ExportTransactionsParams::new(NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid date"));

    // deny_unknown_fields on MoneymoneyTransaction will surface unknown keys as
    // deserialisation errors.
    let response = export_transactions(params).expect("MoneyMoney must be running and unlocked");
    eprintln!(
        "checked {} live transactions for schema conformance",
        response.transactions.len()
    );
}

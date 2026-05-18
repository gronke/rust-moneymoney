//! Validate that the binding's parameter structs match the parameters MoneyMoney
//! documents in its scripting definition file.
//!
//! `MoneyMoney.sdef` is shipped inside MoneyMoney.app and is proprietary — we read
//! it from the local install at runtime rather than vendoring it. The path resolves
//! in this order:
//!
//!   1. `$MONEYMONEY_SDEF_PATH` if set (lets CI or non-default installs point at a
//!      specific copy),
//!   2. `/Applications/MoneyMoney.app/Contents/Resources/MoneyMoney.sdef` (default
//!      install location on macOS).
//!
//! If neither resolves, each test prints a `SKIP:` line and returns — so plain
//! `cargo test` on machines without MoneyMoney stays green.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use sdef::Dictionary;

const DEFAULT_SDEF_PATH: &str = "/Applications/MoneyMoney.app/Contents/Resources/MoneyMoney.sdef";
const SDEF_ENV_VAR: &str = "MONEYMONEY_SDEF_PATH";

/// Returns `Some(path)` if a readable sdef is available, otherwise `None`. Callers
/// emit the SKIP message themselves so the test output names which test was skipped.
fn resolve_sdef_path() -> Option<PathBuf> {
    let candidate = std::env::var_os(SDEF_ENV_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SDEF_PATH));
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

fn read_sdef_or_skip(test_name: &str) -> Option<String> {
    let Some(path) = resolve_sdef_path() else {
        eprintln!(
            "SKIP {test_name}: no MoneyMoney.sdef found. Install MoneyMoney or set \
             {SDEF_ENV_VAR} to point at the file."
        );
        return None;
    };
    match fs::read_to_string(&path) {
        Ok(contents) => Some(contents),
        Err(e) => {
            eprintln!("SKIP {test_name}: failed to read {path:?}: {e}");
            None
        }
    }
}

fn documented_param_names(sdef: &str, command: &str) -> HashSet<String> {
    let dict: Dictionary = sdef
        .parse()
        .expect("MoneyMoney.sdef failed to parse as a Dictionary");
    dict.suites
        .iter()
        .flat_map(|s| &s.commands)
        .find(|c| c.name == command)
        .unwrap_or_else(|| panic!("command '{command}' not found in sdef"))
        .parameters
        .iter()
        .map(|p| p.name.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Every sdef parameter of `export transactions` must map either to a serde field of
/// `ExportTransactionsParams` or to a fixed value the call site hard-codes (`as` is
/// always "plist"). New parameters or renames on MoneyMoney's side fail this test.
#[test]
fn export_transactions_params_in_sync_with_sdef() {
    let Some(sdef) = read_sdef_or_skip("export_transactions_params_in_sync_with_sdef") else {
        return;
    };

    // sdef parameter name → expected serde key on ExportTransactionsParams, or
    // `None` for parameters the call site fixes.
    let mapping: &[(&str, Option<&str>)] = &[
        ("from account", Some("fromAccount")),
        ("from category", Some("fromCategory")),
        ("from date", Some("fromDate")),
        ("to date", Some("toDate")),
        ("as", None), // hard-coded to "plist" in call_action_plist
    ];

    let documented = documented_param_names(&sdef, "export transactions");
    let mapped: HashSet<&str> = mapping.iter().map(|(n, _)| *n).collect();

    let unexpected: Vec<&String> = documented
        .iter()
        .filter(|n| !mapped.contains(n.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "sdef has parameters our mapping doesn't know about — MoneyMoney may have \
         added new parameters that need wiring through ExportTransactionsParams: \
         {unexpected:?}"
    );

    let removed: Vec<&str> = mapping
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| !documented.contains(*n))
        .collect();
    assert!(
        removed.is_empty(),
        "mapping references parameters no longer in the sdef — MoneyMoney may have \
         removed or renamed them: {removed:?}"
    );

    let expected_struct_keys: HashSet<&str> = mapping.iter().filter_map(|(_, k)| *k).collect();
    let struct_keys = export_transactions_params_serde_keys();
    let missing: Vec<&&str> = expected_struct_keys.difference(&struct_keys).collect();
    assert!(
        missing.is_empty(),
        "ExportTransactionsParams missing fields for sdef parameters: {missing:?}"
    );
}

/// Spot-check the other export commands so any new parameter surface is flagged.
#[test]
fn other_export_commands_have_known_param_shape() {
    let Some(sdef) = read_sdef_or_skip("other_export_commands_have_known_param_shape") else {
        return;
    };

    let cases: &[(&str, &[&str])] = &[
        ("export accounts", &[]),
        ("export categories", &[]),
        ("export portfolio", &["from account", "from asset class", "as"]),
    ];

    for (command, expected) in cases {
        let documented = documented_param_names(&sdef, command);
        let expected: HashSet<&str> = expected.iter().copied().collect();
        let actual: HashSet<&str> = documented.iter().map(String::as_str).collect();
        assert_eq!(
            expected, actual,
            "parameter set for '{command}' drifted; update the test or the binding"
        );
    }
}

/// Reflectively enumerate the serde-serialised field names of
/// `ExportTransactionsParams` by round-tripping a fully-populated instance to JSON
/// — keeps the test free of a `schemars`/derived-reflection dependency.
fn export_transactions_params_serde_keys() -> HashSet<&'static str> {
    use chrono::NaiveDate;
    use moneymoney::export_transactions::ExportTransactionsParams;

    let probe = ExportTransactionsParams::new(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
        .to_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
        .from_account("x")
        .from_category("y");

    let value: serde_json::Value = serde_json::to_value(&probe).expect("probe serialises");
    value
        .as_object()
        .expect("object")
        .keys()
        .map(|k| Box::leak(k.clone().into_boxed_str()) as &'static str)
        .collect()
}

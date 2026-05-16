//! `create bank-transfer` CLI against the seeded offline account `test-cash`.
//!
//! Requires MoneyMoney, the `experimental` feature, and `scripts/create_test_accounts.sh`.
//! Passes JSON via a temp file (not stdin) to the `moneymoney` binary.
//! Run: `cargo test -p moneymoney-cli --features experimental -- --ignored`

#![cfg(feature = "experimental")]

use std::fs;
use std::process::Command;

use moneymoney::export_accounts;

#[test]
#[ignore]
fn test_cli_create_bank_transfer_from_test_cash_outbox() {
    let accounts = export_accounts().expect("export_accounts");
    assert!(
        accounts.iter().any(|a| a.name == "test-cash" && !a.group),
        "missing offline account `test-cash` — run scripts/create_test_accounts.sh"
    );

    let json = br#"{"fromAccount":"test-cash","to":"moneymoney test payee","iban":"DE89370400440532013000","amount":0.01,"purpose":"CLI create bank-transfer integration test","into":"outbox"}"#;

    let path = std::env::temp_dir()
        .join(format!("moneymoney-cli-bank-transfer-test-{}.json", std::process::id()));
    fs::write(&path, json).expect("write temp JSON");

    let output = Command::new(env!("CARGO_BIN_EXE_moneymoney"))
        .args(["create", "bank-transfer"])
        .arg(&path)
        .output()
        .expect("run moneymoney create bank-transfer");

    let _ = fs::remove_file(&path);

    assert!(
        output.status.success(),
        "moneymoney create bank-transfer failed (exit={})\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

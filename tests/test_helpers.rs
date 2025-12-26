//! Test helper functions for integration tests

use moneymoney::export_accounts;

pub fn ensure_test_accounts_exist() -> Result<(), String> {
    let accounts = export_accounts::call().map_err(|e| {
        format!(
            "Failed to connect to MoneyMoney. Is it running? Error: {}",
            e
        )
    })?;

    let test_accounts: Vec<_> = accounts
        .iter()
        .filter(|a| a.name.starts_with("test-"))
        .collect();

    if test_accounts.is_empty() {
        return Err(format!(
            "\n\n❌ NO TEST ACCOUNTS FOUND\n\n\
            The roundtrip integration tests require test accounts to be created manually.\n\
            This is a ONE-TIME setup (MoneyMoney's API doesn't support account creation).\n\n\
            Please create these two offline accounts in MoneyMoney:\n\n\
            1. Account name: test-cash\n\
               Type: Cash Account\n\
               Currency: EUR\n\
               Initial Balance: 0.00\n\n\
            2. Account name: test-checking\n\
               Type: Giro/Checking Account\n\
               Currency: EUR\n\
               Initial Balance: 0.00\n\n\
            How to create:\n\
               • Open MoneyMoney\n\
               • File → New Account (⌘N)\n\
               • Select \"Offline Account\"\n\
               • Choose account type\n\
               • Enter name and currency\n\
               • Click Create\n\n\
            After creating accounts, run the tests again.\n\
            Tests use only 'test-' prefixed accounts and won't touch your real data.\n"
        ));
    }

    if test_accounts.len() < 2 {
        return Err(format!(
            "\n\n⚠️  INCOMPLETE TEST SETUP\n\n\
            Found {} test account(s), but need 2:\n{}\n\n\
            Missing accounts:\n{}\n\n\
            Please create the missing accounts in MoneyMoney (see above for instructions).\n",
            test_accounts.len(),
            test_accounts
                .iter()
                .map(|a| format!("  ✓ {}", a.name))
                .collect::<Vec<_>>()
                .join("\n"),
            if !test_accounts.iter().any(|a| a.name == "test-cash") {
                "  ✗ test-cash (Cash Account, EUR)"
            } else {
                ""
            },
        )
        .trim()
        .to_string()
            + &if !test_accounts.iter().any(|a| a.name == "test-checking") {
                "\n  ✗ test-checking (Giro Account, EUR)"
            } else {
                ""
            }
            .to_string());
    }

    println!("✅ Found {} test accounts:", test_accounts.len());
    for account in &test_accounts {
        println!("   • {} ({}, {})", account.name, account.account_type, account.currency);
    }

    Ok(())
}

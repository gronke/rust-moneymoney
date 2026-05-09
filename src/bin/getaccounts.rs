fn main() -> Result<(), moneymoney::Error> {
    // Export all accounts
    let accounts = moneymoney::export_accounts()?;
    for account in accounts {
        println!("{}: {} {} {:?}",
            account.name,
            account.balance.amount,
            account.balance.currency,
            account.r#type

        );
    }

    Ok(())
}
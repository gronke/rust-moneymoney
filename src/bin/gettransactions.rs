use chrono::NaiveDate;
use moneymoney::export_transactions::ExportTransactionsParams;

fn main() -> Result<(), moneymoney::Error> {
    let params =
        ExportTransactionsParams::new(NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date"));
    let response = moneymoney::export_transactions(params)?;
    for i in response.transactions {
        println!("{}", i);
    }
    Ok(())
}

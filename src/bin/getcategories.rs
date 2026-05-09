fn main() -> Result<(), moneymoney::Error> {
    let categories = moneymoney::export_categories()?;
    for category in categories {
        if let Some(budget) = category.budget {
            println!(
                "{}: {} {} budget, {} available",
                category.name, budget.amount, category.currency, budget.available
            );
        }
    }
    Ok(())
}

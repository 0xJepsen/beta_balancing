use anyhow::{Ok, Result};
pub mod portfolio;

#[tokio::main]
async fn main() -> Result<()> {
    let mut portfolio = portfolio::Portfolio::builder().build().await?;

    println!("Positions: {:#?}", portfolio.positions);
    println!("Target weights: {:#?}", portfolio.target_weights);
    println!("Rebalance type: {:#?}", portfolio.rebalance_type);

    let start_time = std::time::Instant::now();
    println!("actual weights:{:#?} ", portfolio.get_actual_weights().await?);
    let duration = start_time.elapsed();
    println!("Time taken to update weights: {:?}", duration);

    println!("Portfolio value: {}", portfolio.get_portfolio_value());
    Ok(())
}

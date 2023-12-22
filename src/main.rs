use anyhow::{Ok, Result};
pub mod portfolio;

#[tokio::main]
async fn main() -> Result<()> {
    let mut portfolio = portfolio::Portfolio::builder().build().await?;

    println!("Positions: {:#?}", portfolio.positions);
    println!("Target weights: {:#?}", portfolio.target_weights);
    println!("Rebalance type: {:#?}", portfolio.rebalance_type);

    println!("actual weights:{:#?} ", portfolio.get_actual_weights().await?);

    Ok(())
}

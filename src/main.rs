use anyhow::{Ok, Result};
pub mod portfolio;
#[allow(dead_code)]

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

    let historical_prices = get_historical_daily_prices(5, "ethereum").await?;
    println!("Historical prices: {:?}", historical_prices);

    Ok(())
}

pub async fn get_historical_daily_prices(number_of_days: i64, id: &str) -> Result<Vec<f64>> {
    let id = id.to_string(); // Clone id here
    let eth_historical_price = tokio::task::spawn_blocking(move || {
        rust_gecko::coins::market_chart(
            &id,
            "usd",
            (number_of_days - 1).to_string().as_str(),
            Some("daily"),
        )
    }).await?;
    let json = eth_historical_price.json.clone().unwrap();
    // can also parse the daily market caps and total volumes from this repsonse
    let prices: Vec<_> = json
        .get("prices")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_array().unwrap()[1].as_f64().unwrap())
        .collect();
    Ok(prices)
}

pub async fn get_eth_price() -> Result<f64> {
    let res = tokio::task::spawn_blocking(move || {
        rust_gecko::simple::price(vec!["ethereum"], vec!["usd"], None, None, None, None)
    }).await?;
    match res.json {
        Some(json) => {
            let eth_price = json.get("ethereum").and_then(|eth| eth.get("usd")).unwrap();
            let eth_price = eth_price.as_f64().unwrap_or(0.0);
            Ok(eth_price)
        }
        None => Err(anyhow::Error::msg("No data received")),
    }
}

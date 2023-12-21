use std::collections::hash_map::HashMap;

use RustQuant::{self, data::{YahooFinanceData, YahooFinanceReader}, portfolio::{self, Position}};
use anyhow::{Result, Ok};
use tokio;
use yahoo_finance_api as yahoo;

struct Portfolio {
    positions: HashMap<String, f32>,
    value_over_time: Vec<f32>,
    rebalance_frequency: u32,
    rebalance_threshold: Option<f32>,
}


#[tokio::main]
async fn main() -> Result<()> {

    let portfolio = get_portfolio();
    let prices = get_prices(&portfolio).await;

    let total_value: f32 = portfolio.iter()
        .map(|(symbol, weight)| weight * prices.get(symbol).unwrap_or(&0.0))
        .sum();
    println!("Total value of the portfolio: ${}", total_value);
    Ok(())

}

async fn get_prices(portfolio: &HashMap<String, f32>) -> HashMap<String, f32> {
    let provider = yahoo::YahooConnector::new();
    let mut prices = HashMap::new();

    for (symbol, _) in portfolio {
        let response = provider.get_latest_quotes(symbol, "1d").await.unwrap();
        let quote = response.last_quote().unwrap();
        prices.insert(symbol.clone(), quote.close as f32);
    }
    prices
}

pub fn get_portfolio() -> HashMap<String, f32> {
    let mut portfolio = HashMap::new();
    portfolio.insert("COIN".to_string(), 0.15);
    portfolio.insert("NVDA".to_string(), 0.20);
    portfolio.insert("GLDM".to_string(), 0.10);
    portfolio.insert("SPY".to_string(), 0.30);
    portfolio.insert("ENPH".to_string(), 0.10);
    portfolio.insert("QCLN".to_string(), 0.10);
    portfolio.insert("MSTR".to_string(), 0.025);
    portfolio.insert("MARA".to_string(), 0.025);
    let total: f32 = portfolio.values().sum();
    assert!((total - 1.0).abs() < 0.01, "Values in portfolio do not sum to 1");

    portfolio
}
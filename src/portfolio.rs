use std::collections::HashMap;

use anyhow::{Ok, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use polars::prelude::*;

use crate::assets::{Stock, Crypto, Asset};

pub struct Portfolio {
    // asset and wieght
    pub positions: (Vec<Stock>, Vec<Crypto>),
    // maybe we want time series of pvf
    // value_over_time: Vec<f32>,
    // target weights
    pub target_weights: HashMap<String, f32>,
    // rebalance type
    pub rebalance_type: RebalanceType,
    // reblance threshold
    pub rebalance_threshold: Option<f32>,
}
impl Portfolio {
    pub fn builder() -> PortfolioBuilder {
        PortfolioBuilder::new()
    }

    pub fn get_portfolio_value(&self) -> f64 {
        let stocks_value = self.positions.0
            .iter()
            .fold(0.0, |acc, x| acc + (x.last_price * x.amount_held));
        let cryptos_value = self.positions.1
            .iter()
            .fold(0.0, |acc, x| acc + (x.last_price * x.amount_held));
        stocks_value + cryptos_value
    }

    pub async fn get_actual_weights(&mut self) -> Result<DataFrame> {
        self.update_prices().await?;
        let mut actual_weights = HashMap::new();
        let total_value = self.positions.0.iter().fold(0.0, |acc, x| acc + x.last_price * x.amount_held)
        + self.positions.1.iter().fold(0.0, |acc, x| acc + x.last_price * x.amount_held);

        for asset in self.positions.0.iter().map(|x| x as &dyn Asset).chain(self.positions.1.iter().map(|x| x as &dyn Asset)) {
            let weight = asset.last_price() * asset.amount_held() / total_value;
            actual_weights.insert(asset.ticker(), weight);
        }

        let total_weight: f64 = actual_weights.values().sum();
        assert!(
            (total_weight - 1.0).abs() < 1e-8,
            "Weights do not add up to 1"
        );
        let df = self.weights_to_dataframe(actual_weights)?;
        Ok(df)
    }
    pub fn weights_to_dataframe(&self, weights: HashMap<String, f64>) -> Result<DataFrame> {
        let tickers: Vec<_> = weights.keys().cloned().collect();
        let weights: Vec<_> = weights.values().cloned().collect();
        Ok(df!(
            "ticker" => tickers,
            "weight" => weights
        )?)
    }

    async fn update_prices(&mut self) -> Result<()> {
        self.update_stock_prices().await?;
        self.update_crypto_prices().await?;
        Ok(())
    }
    async fn update_stock_prices(&mut self) -> Result<()> {
        let mut futures: FuturesUnordered<_> = self
            .positions
            .0
            .iter_mut()
            .map(|asset| asset.fetch_price())
            .collect();
        while let Some(result) = futures.next().await {
            result?;
        }
        Ok(())
    }
    async fn update_crypto_prices(&mut self) -> Result<()> {
        let mut futures: FuturesUnordered<_> = self
            .positions
            .1
            .iter_mut()
            .map(|asset| asset.fetch_price())
            .collect();
        while let Some(result) = futures.next().await {
            result?;
        }
        Ok(())
    }
}

pub struct PortfolioBuilder {
    positions: Vec<Stock>,
    target_weights: HashMap<String, f32>,
    rebalance_type: RebalanceType,
    rebalance_threshold: Option<f32>,
}

impl Default for PortfolioBuilder {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            target_weights: HashMap::new(),
            rebalance_type: RebalanceType::None,
            rebalance_threshold: None,
        }
    }
}

impl PortfolioBuilder {
    pub fn new() -> PortfolioBuilder {
        PortfolioBuilder::default()
    }

    pub async fn build(self) -> Result<Portfolio> {
        if self.positions.is_empty() {
            let stock = vec![
                Stock::new(COIN, 10.0).await?,
                Stock::new(NVDA, 2.0).await?,
                Stock::new(GLDM, 4.0).await?,
                Stock::new(SPY, 1.0).await?,
                Stock::new(ENPH, 3.0).await?,
                Stock::new(APPL, 1.5).await?,
                Stock::new(MSFT, 0.38).await?,
            ];
            let crypto = vec![
                Crypto::new("ethereum", "ETH", 10.0).await?,
            ];
            Ok(Portfolio {
                positions: (stock, crypto),
                target_weights: load_weights(),
                rebalance_type: RebalanceType::Threshold(REBALANCE_THRESHOLD),
                rebalance_threshold: self.rebalance_threshold,
            })
        } else {
            Ok(Portfolio {
                positions: (self.positions, Vec::new()),
                target_weights: self.target_weights,
                rebalance_type: self.rebalance_type,
                rebalance_threshold: self.rebalance_threshold,
            })
        }
    }

    pub async fn add_asset(mut self, ticker: &str, amount: f64) -> Self {
        let asset = Stock::new(ticker, amount).await.unwrap();
        self.positions.push(asset);
        self
    }

    pub fn rebalance_type(mut self, rebalance_type: RebalanceType) -> Self {
        self.rebalance_type = rebalance_type;
        self
    }

    pub fn rebalance_threshold(mut self, threshold: Option<f32>) -> Self {
        self.rebalance_threshold = threshold;
        self
    }
}
pub enum RebalanceType {
    Threshold(f32),
    Frequency(u32),
    ThresholdAndFrequency(f32, u32),
    None,
}
impl std::fmt::Debug for RebalanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RebalanceType::Threshold(t) => write!(f, "Threshold({})", t),
            RebalanceType::Frequency(u) => write!(f, "Frequency({})", u),
            RebalanceType::ThresholdAndFrequency(t, u) => {
                write!(f, "ThresholdAndFrequency({}, {})", t, u)
            }
            RebalanceType::None => write!(f, "None"),
        }
    }
}


fn load_weights() -> HashMap<String, f32> {
    let mut map = HashMap::new();
    map.insert(COIN.to_string(), 0.15);
    map.insert(NVDA.to_string(), 0.20);
    map.insert(GLDM.to_string(), 0.10);
    map.insert(SPY.to_string(), 0.30);
    map.insert(ENPH.to_string(), 0.10);
    map.insert(QCLN.to_string(), 0.10);
    map.insert(MSTR.to_string(), 0.025);
    map.insert(MARA.to_string(), 0.025);
    map
}

#[allow(dead_code)]
const REBALANCE_FREQUENCY: u32 = 30;
const REBALANCE_THRESHOLD: f32 = 0.05;

const MSFT: &str = "MSFT";
#[allow(dead_code)]
const AMD: &str = "AMD";
const APPL: &str = "AAPL";
const COIN: &str = "COIN";
const NVDA: &str = "NVDA";
const GLDM: &str = "GLDM";
const SPY: &str = "SPY";
const ENPH: &str = "ENPH";
const QCLN: &str = "QCLN";
const MSTR: &str = "MSTR";
const MARA: &str = "MARA";

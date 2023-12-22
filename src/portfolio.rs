use std::collections::HashMap;

use polars::prelude::*;
use yahoo_finance_api::YahooConnector;
use anyhow::{Ok, Result};

pub struct Portfolio {
    // asset and wieght
    pub positions: Vec<Asset>,
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

    pub async fn get_actual_weights(&mut self) -> Result<DataFrame> {
        self.update_prices().await?;
        let mut actual_weights = HashMap::new();
        let total_value = self.positions.iter().fold(0.0, |acc, x| acc + x.last_price);
        for asset in &self.positions {
            let weight = asset.last_price / total_value;
            actual_weights.insert(asset.ticker.clone(), weight);
        }
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
        for asset in &mut self.positions {
            asset.fetch_price().await?;
        }
        Ok(())
    }
}

pub struct PortfolioBuilder {
    positions: Vec<Asset>,
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
            let positions = vec![
                Asset::new(COIN).await?,
                Asset::new(NVDA).await?,
                Asset::new(GLDM).await?,
                Asset::new(SPY).await?,
                Asset::new(ENPH).await?,
                Asset::new(QCLN).await?,
                Asset::new(MSTR).await?,
                Asset::new(MARA).await?,
            ];
            Ok(Portfolio {
                positions,
                target_weights: load_weights(),
                rebalance_type: RebalanceType::Threshold(REBALANCE_THRESHOLD),
                rebalance_threshold: self.rebalance_threshold,
            })
        } else {
            Ok(Portfolio {
                positions: self.positions,
                target_weights: self.target_weights,
                rebalance_type: self.rebalance_type,
                rebalance_threshold: self.rebalance_threshold,
            })
        }
    }

    pub async fn add_asset(mut self, ticker: &str) -> Self {
        let asset = Asset::new(ticker).await.unwrap();
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

pub struct Asset {
    pub ticker: String,
    pub client: YahooConnector,
    pub last_price: f64,
    pub name: String,
}

impl std::fmt::Debug for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Asset {{ ticker: {}, name: {}, last_price: {} }}",
            self.ticker, self.name, self.last_price
        )
    }
}

impl Asset {
    async fn new(ticker: &str) -> Result<Self> {
        let client = YahooConnector::new();
        let res = client.get_latest_quotes(ticker, "1d").await?;
        let currency = res.metadata().unwrap().currency;
        let last_price = res.last_quote()?.close;

        Ok(Self {
            ticker: ticker.to_string(),
            client,
            name: currency,
            last_price,
        })
    }

    #[allow(dead_code)]
    async fn fetch_price(&mut self) -> Result<()> {
        let res = self.client.get_latest_quotes(&self.ticker, "1d").await?;
        self.last_price = res.last_quote()?.close;
        Ok(())
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

const COIN: &str = "COIN";
const NVDA: &str = "NVDA";
const GLDM: &str = "GLDM";
const SPY: &str = "SPY";
const ENPH: &str = "ENPH";
const QCLN: &str = "QCLN";
const MSTR: &str = "MSTR";
const MARA: &str = "MARA";

use std::collections::HashMap;

use anyhow::{Ok, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use polars::prelude::*;

use crate::assets::{Asset, Crypto, Stock};
use crate::safe_money::USD;

pub struct Portfolio {
    // asset and wieght
    pub positions: (Vec<Stock>, Vec<Crypto>),
    // maybe we want time series of pvf
    // value_over_time: Vec<f32>,
    // target weights
    pub target_weights: HashMap<String, f64>,
    // Actual weights
    pub actual_weights: HashMap<String, f64>,
    // rebalance type
    pub rebalance_type: RebalanceType,
    // reblance threshold
    pub rebalance_threshold: Option<f64>,
    // cash on hand
    pub cash: USD,
}
impl Portfolio {
    pub fn builder() -> PortfolioBuilder {
        PortfolioBuilder::new()
    }

    pub fn get_portfolio_value(&self) -> USD {
        let stocks_value = self
            .positions
            .0
            .iter()
            .fold(0.0, |acc, x| acc + (x.last_price.amount * x.amount_held));
        let cryptos_value = self
            .positions
            .1
            .iter()
            .fold(0.0, |acc, x| acc + (x.last_price * x.amount_held));
        stocks_value + cryptos_value + self.cash
    }

    pub fn get_actual_weights(&mut self) -> Result<HashMap<String, f64>> {
        let mut actual_weights: HashMap<String, f64> = HashMap::new();
        let total_value = self.get_portfolio_value();

        for asset in self
            .positions
            .0
            .iter()
            .map(|x| x as &dyn Asset)
            .chain(self.positions.1.iter().map(|x| x as &dyn Asset))
        {
            let weight = (USD::new(asset.last_price().amount * asset.amount_held()) / total_value).amount;
            actual_weights.insert(asset.ticker(), weight);
        }

        // Add cash weight
        let cash_weight = (self.cash / total_value).amount;
        actual_weights.insert("CASH".to_string(), cash_weight);

        let total_weight: f64 = actual_weights.values().sum();
        assert!(
            (total_weight - 1.0).abs() < 1e-8,
            "Weights do not add up to 1"
        );
        self.actual_weights = actual_weights.clone();
        // let df = self.weights_to_dataframe(actual_weights)?;
        Ok(actual_weights)
    }
    pub fn weights_to_dataframe(&self, weights: HashMap<String, f64>) -> Result<DataFrame> {
        let tickers: Vec<_> = weights.keys().cloned().collect();
        let weights: Vec<_> = weights.values().cloned().collect();
        Ok(df!(
            "ticker" => tickers,
            "weight" => weights
        )?)
    }

    #[allow(dead_code)]
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

    pub fn rebalance(&mut self) -> Result<()> {
        let original_pvf = self.get_portfolio_value();
        let target_weights = &self.target_weights;
        let actual_weights = &self.actual_weights;

        let mut trades = Vec::new();

        for asset in &self.positions.0 {
            if let Some(target_weight) = target_weights.get(&asset.ticker) {
                let actual_weight = actual_weights.get(&asset.ticker()).unwrap_or(&0.0);
                let target_quantity = original_pvf.amount * (*target_weight);
                let actual_quantity = original_pvf.amount * (*actual_weight);
                let amount_to_trade = target_quantity - actual_quantity;

                if amount_to_trade.abs() > self.rebalance_threshold.unwrap_or(0.0) {
                    let price = asset.last_price();
                    let quantity_to_trade = amount_to_trade / price;
                    trades.push((quantity_to_trade, asset.ticker.clone()));
                }
            }
        }

        for (quantity_to_trade, ticker) in trades {
            // If actual weight is higher than target weight, sell to reach target weight
            if quantity_to_trade < 0.0 {
                self.paper_sell(quantity_to_trade.abs(), &ticker)?;
            }
            // If actual weight is lower than target weight, buy to reach target weight
            else if quantity_to_trade > 0.0 {
                self.paper_buy(quantity_to_trade.abs(), &ticker)?;
            }
        }
        self.reinvest()?;
        let new_pvf = self.get_portfolio_value();
        assert!(new_pvf == original_pvf);
        Ok(())
    }

    fn reinvest(&mut self) -> Result<()> {
        let excess_cash = self.cash;
        let num_assets = self.positions.0.len() as f64;
        let cash_per_asset = USD::new(excess_cash.amount / num_assets);

        let mut quantities_to_buy = Vec::new();

        for asset in &self.positions.0 {
            if cash_per_asset.amount > 0.0 {
                let quantity_to_buy = 
                (cash_per_asset / asset.last_price()).amount;
                quantities_to_buy.push((quantity_to_buy, asset.ticker.clone()));
            }
        }

        for (quantity_to_buy, ticker) in quantities_to_buy {
            self.paper_buy(quantity_to_buy, &ticker)?;
        }

        Ok(())
    }

    pub fn paper_buy(&mut self, quantity: f64, ticker: &str) -> Result<()> {
        if quantity < 0.0 {
            return Err(anyhow::Error::msg("Quantity must be positive"));
        }
        let asset = self
            .positions
            .0
            .iter()
            .find(|x| x.ticker == ticker)
            .unwrap();
        if USD::new(quantity * asset.last_price.amount) > self.cash {
            return Err(anyhow::Error::msg("Not enough cash"));
        } else {
            self.cash -= USD::new(quantity * asset.last_price.amount);
            let asset = self
                .positions
                .0
                .iter_mut()
                .find(|x| x.ticker == ticker)
                .unwrap();
            asset.amount_held += quantity;
        }
        Ok(())
    }

    pub fn paper_sell(&mut self, quantity: f64, ticker: &str) -> Result<()> {
        if quantity < 0.0 {
            return Err(anyhow::Error::msg("Quantity must be positive"));
        }
        let asset = self
            .positions
            .0
            .iter()
            .find(|x| x.ticker == ticker)
            .unwrap();
        if quantity > asset.amount_held {
            return Err(anyhow::Error::msg("Not enough assets to sell"));
        } else {
            self.cash += USD::new(quantity * asset.last_price.amount);
            let asset = self
                .positions
                .0
                .iter_mut()
                .find(|x| x.ticker == ticker)
                .unwrap();
            asset.amount_held -= quantity;
        }
        Ok(())
    }
}

pub struct PortfolioBuilder {
    positions: Vec<Stock>,
    target_weights: HashMap<String, f64>,
    actual_weights: HashMap<String, f64>,
    rebalance_type: RebalanceType,
    rebalance_threshold: Option<f64>,
}

impl Default for PortfolioBuilder {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            target_weights: HashMap::new(),
            actual_weights: HashMap::new(),
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
            let crypto = vec![];
            let actual_weights = HashMap::new();
            Ok(Portfolio {
                positions: (stock, crypto),
                target_weights: self.load_target_weights(),
                actual_weights,
                rebalance_type: RebalanceType::Threshold(REBALANCE_THRESHOLD),
                rebalance_threshold: self.rebalance_threshold,
                cash: 0.0.into(),
            })
        } else {
            Ok(Portfolio {
                positions: (self.positions, Vec::new()),
                target_weights: self.target_weights,
                actual_weights: self.actual_weights,
                rebalance_type: self.rebalance_type,
                rebalance_threshold: self.rebalance_threshold,
                cash: 0.0.into(),
            })
        }
    }
    fn load_target_weights(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        map.insert(COIN.to_string(), 0.15);
        map.insert(NVDA.to_string(), 0.20);
        map.insert(GLDM.to_string(), 0.10);
        map.insert(SPY.to_string(), 0.30);
        map.insert(ENPH.to_string(), 0.10);
        map.insert(QCLN.to_string(), 0.10);
        map.insert(MSTR.to_string(), 0.025);
        map.insert(MARA.to_string(), 0.025);
        map.insert("USD".to_string(), 0.0);
        map
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

    pub fn rebalance_threshold(mut self, threshold: Option<f64>) -> Self {
        self.rebalance_threshold = threshold;
        self
    }
}
pub enum RebalanceType {
    Threshold(f64),
    Frequency(u32),
    ThresholdAndFrequency(f64, u32),
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

#[allow(dead_code)]
const REBALANCE_FREQUENCY: u32 = 30;
const REBALANCE_THRESHOLD: f64 = 0.05;

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

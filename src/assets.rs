use yahoo_finance_api::YahooConnector;
use anyhow::Result;

pub trait Asset {
    fn last_price(&self) -> f64;
    fn amount_held(&self) -> f64;
    fn ticker(&self) -> String;
}

impl Asset for Stock {
    fn last_price(&self) -> f64 { self.last_price }
    fn amount_held(&self) -> f64 { self.amount_held }
    fn ticker(&self) -> String { self.ticker.clone() }
}

impl Asset for Crypto {
    fn last_price(&self) -> f64 { self.last_price }
    fn amount_held(&self) -> f64 { self.amount_held }
    fn ticker(&self) -> String { self.token.clone() }
}
pub struct Stock {
    pub ticker: String,
    pub amount_held: f64,
    pub client: YahooConnector,
    pub last_price: f64,
    pub name: String,
}

impl std::fmt::Debug for Stock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Asset {{ ticker: {}, name: {}, last_price: {} }}",
            self.ticker, self.name, self.last_price
        )
    }
}

#[allow(dead_code)]
impl Stock {
    pub async fn new(ticker: &str, ammount: f64) -> Result<Self> {
        let client = YahooConnector::new();
        let res = client.get_latest_quotes(ticker, "1d").await?;
        let currency = res.metadata().unwrap().currency;
        let last_price = res.last_quote()?.close;

        Ok(Self {
            amount_held: ammount,
            ticker: ticker.to_string(),
            client,
            name: currency,
            last_price,
        })
    }

    #[allow(dead_code)]
    pub async fn fetch_price(&mut self) -> Result<()> {
        let res = self.client.get_latest_quotes(&self.ticker, "1d").await?;
        self.last_price = res.last_quote()?.close;
        Ok(())
    }
}

pub struct Crypto {
    pub name: String,
    pub amount_held: f64,
    pub last_price: f64,
    pub token: String,
}

impl std::fmt::Debug for Crypto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ name: {}, last_price: {} }}",
            self.name, self.last_price
        )
    }
}

#[allow(dead_code)]
impl Crypto {
    pub async fn new(name: &str, token: &str, ammount: f64) -> Result<Self> {
        let mut s = Self {
            name: name.to_owned(),
            amount_held: ammount,
            last_price: 0.0,
            token: token.to_owned(),
        };

        let last_price = s.fetch_price().await?;

        Ok(Self {
            last_price,
            ..s
        })

    }

    pub async fn fetch_price(&mut self) -> Result<f64> {
        let res_owned_name = self.name.clone();
        let res_owned_name_clone = res_owned_name.clone(); 
        let res = tokio::task::spawn_blocking(move || {
            rust_gecko::simple::price(vec![&res_owned_name], vec!["usd"], None, None, None, None)
        }).await?;
        match res.json {
            Some(json) => {
                let eth_price = json.get(res_owned_name_clone).and_then(|eth| eth.get("usd")).unwrap();
                let eth_price = eth_price.as_f64().unwrap_or(0.0);
                Ok(eth_price)
            }
            None => Err(anyhow::Error::msg("No data received")),
        }
    }
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
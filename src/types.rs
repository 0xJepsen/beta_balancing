

pub struct Portfolio {
    positions: HashMap<String, f32>,
    value_over_time: Vec<f32>,
    rebalance_type: RebalanceType,
    rebalance_threshold: Option<f32>,
}


pub struct Asset {
    ticker: String,
}

impl Instrument for Asset {
    fn price(&self) -> f64 {
        0.0
    }

    fn error(&self) -> Option<f64> {
        None
    }

    fn valuation_date(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    fn instrument_type(&self) -> &'static str {
        "Asset"
    }
}

pub enum RebalanceType {
    Threshold(f32),
    Frequency(u32),
    ThresholdAndFrequency(f32, u32),
    None,
}
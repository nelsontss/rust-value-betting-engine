use chrono::{DateTime, Utc};

use crate::domain::Market;

#[derive(Debug, PartialEq)]
pub struct MarketDataPoint {
    market: Market,
    datetime: DateTime<Utc>,
}

impl MarketDataPoint {
    pub fn new(market: Market) -> Self {
        MarketDataPoint {
            market,
            datetime: Utc::now(),
        }
    }

    pub fn market(&self) -> &Market {
        &self.market
    }

    pub fn datetime(&self) -> &DateTime<Utc> {
        &self.datetime
    }
}

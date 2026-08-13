use crate::domain::services::backtesting::PriceCandle;
use crate::infrastructure::repositories::MarketRow;

#[derive(Debug, Clone)]
pub struct Signal {
    pub market_id: String,
    pub entry_time: i64,
    pub entry_price: f64,
    pub exit_time: i64,
    pub exit_price: f64,
}

pub trait Strategy {
    fn name(&self) -> &str;
    fn evaluate(&self, market: &MarketRow, candles: &[PriceCandle]) -> Vec<Signal>;
}

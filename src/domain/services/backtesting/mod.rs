pub mod draw_value;
pub mod execution;
pub mod metrics;
pub mod strategy;

pub use self::draw_value::DrawValueStrategy;
pub use self::execution::TradeSimulator;
pub use self::metrics::BacktestMetrics;
pub use self::strategy::{Signal, Strategy};

#[derive(Debug, Clone)]
pub struct PriceCandle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

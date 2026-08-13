use crate::domain::services::backtesting::{PriceCandle, Signal, Strategy};
use crate::infrastructure::config::trade_config::TradeConfig;
use crate::infrastructure::repositories::MarketRow;

pub struct DrawValueStrategy {
    pub resolution_minutes: u32,
}

impl DrawValueStrategy {
    pub fn new(resolution_minutes: u32) -> Self {
        Self { resolution_minutes }
    }

    fn to_f64(d: polymarket_client_sdk_v2::types::Decimal) -> f64 {
        d.to_string().parse().unwrap_or(0.0)
    }
}

impl Strategy for DrawValueStrategy {
    fn name(&self) -> &str {
        "draw-value"
    }

    fn evaluate(&self, market: &MarketRow, candles: &[PriceCandle]) -> Vec<Signal> {
        let start_str = match &market.match_start {
            Some(s) => s,
            None => return vec![],
        };

        let start_ms = match chrono::DateTime::parse_from_rfc3339(start_str) {
            Ok(dt) => dt.timestamp_millis(),
            Err(_) => return vec![],
        };

        let entry_offset_ms = TradeConfig::BUY_OFFSET.num_milliseconds();
        let exit_offset_ms = TradeConfig::SELL_OFFSET.num_milliseconds();
        let exit_ms = start_ms + exit_offset_ms;

        let min_price = Self::to_f64(TradeConfig::MIN_PRICE);
        let max_price = Self::to_f64(TradeConfig::MAX_PRICE);

        let exit_price = match candles.iter().find(|c| c.timestamp >= exit_ms) {
            Some(c) => c.close,
            None => return vec![],
        };

        for candle in candles.iter() {
            if candle.timestamp >= start_ms {
                break;
            }
            if candle.timestamp < start_ms - entry_offset_ms {
                continue;
            }

            let price = candle.close - 0.01;
            if price >= min_price && price <= max_price {
                return vec![Signal {
                    market_id: market.id.clone(),
                    entry_time: candle.timestamp,
                    entry_price: price,
                    exit_time: exit_ms,
                    exit_price,
                }];
            }
        }

        vec![]
    }
}

use std::collections::HashMap;

use crate::domain::services::backtesting::{BacktestMetrics, DrawValueStrategy, PriceCandle, TradeSimulator};
use crate::infrastructure::repositories::PolymarketRepository;

pub struct BacktestConfig {
    pub resolution_minutes: u32,
    pub from: Option<String>,
    pub to: Option<String>,
}

pub struct BacktestRunner;

impl BacktestRunner {
    pub async fn run(
        repo: &PolymarketRepository,
        config: &BacktestConfig,
    ) -> Result<BacktestMetrics, Box<dyn std::error::Error>> {
        let markets = repo.get_draw_markets().await?;

        let mut candles_map: HashMap<String, Vec<PriceCandle>> = HashMap::new();
        for market in &markets {
            let rows = repo.get_candles_for_market(&market.id).await?;
            candles_map.insert(
                market.id.clone(),
                rows.into_iter()
                    .map(|c| PriceCandle {
                        timestamp: c.timestamp,
                        open: c.open,
                        high: c.high,
                        low: c.low,
                        close: c.close,
                        volume: c.volume,
                    })
                    .collect(),
            );
        }

        let strategy = DrawValueStrategy::new(config.resolution_minutes);

        let metrics = TradeSimulator::run(&strategy, &markets, |market_id| {
            candles_map.remove(market_id).unwrap_or_default()
        });

        Ok(metrics)
    }
}

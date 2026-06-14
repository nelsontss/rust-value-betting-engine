use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use serde::Serialize;

use crate::{
    domain::entities::{MarketDataPoint, MarketType},
    infrastructure::server::dto::market_response::MarketResponse,
};

#[derive(Serialize)]
pub struct MarketDataPointResponse {
    market: MarketResponse,
    updated_at: String,
}

impl From<&Arc<MarketDataPoint>> for MarketDataPointResponse {
    fn from(value: &Arc<MarketDataPoint>) -> Self {
        MarketDataPointResponse {
            market: MarketResponse::from(value.market()),
            updated_at: value.datetime().to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
pub struct MarketHistoryResponse {
    game_id: String,
    markets_history: HashMap<MarketType, Vec<MarketDataPointResponse>>,
}

impl From<(&str, &DashMap<MarketType, Vec<Arc<MarketDataPoint>>>)> for MarketHistoryResponse {
    fn from((game_id, markets): (&str, &DashMap<MarketType, Vec<Arc<MarketDataPoint>>>)) -> Self {
        MarketHistoryResponse {
            game_id: game_id.to_string(),
            markets_history: markets
                .iter()
                .map(|m| {
                    let market_type = m.key();
                    let data_points = m
                        .value()
                        .iter()
                        .map(|dp| MarketDataPointResponse::from(dp))
                        .collect();

                    (market_type.clone(), data_points)
                })
                .collect(),
        }
    }
}

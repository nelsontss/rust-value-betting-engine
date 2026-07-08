use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use serde::Serialize;

use crate::{
    domain::entities::{MarketDataPoint, MarketType},
    infrastructure::server::dto::market_response::MarketResponse,
};

#[derive(Serialize)]
pub struct MarketDataPointResponse {
    game_id: String,
    market: MarketResponse,
    timestamp: String,
}

impl From<(&str, &Arc<MarketDataPoint>)> for MarketDataPointResponse {
    fn from((game_id, value): (&str, &Arc<MarketDataPoint>)) -> Self {
        MarketDataPointResponse {
            game_id: game_id.to_string(),
            market: MarketResponse::from(value.market()),
            timestamp: value.datetime().to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
pub struct MarketHistoryResponse {
    game_id: String,
    markets_by_type: HashMap<String, Vec<MarketDataPointResponse>>,
}

impl From<(&str, &DashMap<MarketType, Vec<Arc<MarketDataPoint>>>)> for MarketHistoryResponse {
    fn from((game_id, markets): (&str, &DashMap<MarketType, Vec<Arc<MarketDataPoint>>>)) -> Self {
        let mut markets_response: HashMap<String, Vec<MarketDataPointResponse>> = HashMap::new();
        for entry_ref in markets.iter() {
            let key = entry_ref.key().variant_name().to_string();
            let points: Vec<MarketDataPointResponse> = entry_ref
                .value()
                .iter()
                .map(|dp| MarketDataPointResponse::from((game_id, dp)))
                .collect();
            markets_response.entry(key).or_default().extend(points);
        }

        MarketHistoryResponse {
            game_id: game_id.to_string(),
            markets_by_type: markets_response,
        }
    }
}

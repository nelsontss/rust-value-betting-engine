use std::{collections::HashMap, sync::Arc};

use tokio::sync::broadcast::{Receiver, Sender};

use crate::{
    domain::{
        Market,
        entities::{MarketDataPoint, MarketType},
    },
    infrastructure::repositories::game_repository::GameRepository,
    shared::error::Result,
};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct MarketService {
    game_repository: Arc<GameRepository>,
    event_tx: Sender<(String, Arc<MarketDataPoint>)>,
}

impl MarketService {
    pub fn new(game_repository: Arc<GameRepository>) -> Self {
        MarketService {
            game_repository,
            event_tx: Sender::new(20),
        }
    }

    pub fn send_new_market_update(&self, game_id: &str, markets: &HashMap<MarketType, Market>) {
        markets.iter().for_each(|(_, market)| {
            let market_data_point = Arc::new(MarketDataPoint::new(market.clone()));

            let _ = self
                .event_tx
                .send((game_id.to_string(), Arc::clone(&market_data_point)));
        });
    }

    pub async fn get_game_markets_history(
        &self,
        game_id: &str,
    ) -> Result<Option<HashMap<MarketType, Vec<Arc<MarketDataPoint>>>>> {
        let data_points = self
            .game_repository
            .get_game_markets_history(game_id)
            .await?;

        if data_points.is_empty() {
            return Ok(None);
        }

        let mut market_data_points_by_type: HashMap<MarketType, Vec<Arc<MarketDataPoint>>> =
            HashMap::new();

        for data_point in data_points {
            let market_type = MarketType::from(data_point.market());
            market_data_points_by_type
                .entry(market_type)
                .or_default()
                .push(Arc::new(data_point));
        }

        Ok(Some(market_data_points_by_type))
    }

    pub fn subscribe_to_game_market_updates(&self) -> Receiver<(String, Arc<MarketDataPoint>)> {
        self.event_tx.subscribe()
    }
}

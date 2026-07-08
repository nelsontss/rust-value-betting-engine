use std::{collections::HashMap, sync::Arc};

use dashmap::{DashMap, mapref::one::Ref};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::domain::{
    Market,
    entities::{MarketDataPoint, MarketType},
};

#[derive(Debug)]
pub struct MarketHistoryService {
    markets_history: DashMap<String, DashMap<MarketType, Vec<Arc<MarketDataPoint>>>>,
    event_tx: Sender<(String, Arc<MarketDataPoint>)>,
}

impl MarketHistoryService {
    pub fn update_market_history(&self, game_id: &str, markets: &HashMap<MarketType, Market>) {
        markets.iter().for_each(|(market_type, market)| {
            let market_data_point = Arc::new(MarketDataPoint::new(market.clone()));

            let mut saved = false;

            self.markets_history
                .entry(game_id.to_string())
                .or_insert_with(DashMap::new)
                .entry(market_type.clone())
                .and_modify(|data_points| {
                    let latest_data_point_market =
                        data_points.last().expect("no datapoint in array").market();

                    if latest_data_point_market != market_data_point.market() {
                        data_points.push(Arc::clone(&market_data_point));
                        saved = true;
                    }
                })
                .or_insert_with(|| {
                    saved = true;
                    vec![Arc::clone(&market_data_point)]
                });

            if saved {
                let _ = self
                    .event_tx
                    .send((game_id.to_string(), Arc::clone(&market_data_point)));
            }
        });
    }

    pub fn get_game_history(
        &'_ self,
        game_id: &str,
    ) -> Option<Ref<'_, String, DashMap<MarketType, Vec<Arc<MarketDataPoint>>>>> {
        self.markets_history.get(game_id)
    }

    pub fn get_game_market_history(
        &self,
        game_id: String,
        market_type: MarketType,
    ) -> Option<&Vec<MarketDataPoint>> {
        if let Some(game_markets) = self.markets_history.get(&game_id)
            && let Some(markets) = game_markets.get(&market_type)
        {
            Some(markets);
        }

        None
    }

    pub fn subscribe_to_game_history_updates(&self) -> Receiver<(String, Arc<MarketDataPoint>)> {
        self.event_tx.subscribe()
    }
}

impl Default for MarketHistoryService {
    fn default() -> Self {
        MarketHistoryService {
            markets_history: DashMap::new(),
            event_tx: Sender::new(20),
        }
    }
}

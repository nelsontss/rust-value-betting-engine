use std::{
    collections::HashMap,
    fmt::{self},
    sync::Arc,
};

use chrono::NaiveDateTime;
use dashmap::DashMap;
use tokio::sync::broadcast::{self, Receiver};

use crate::domain::{
    Market, Platform,
    entities::{Arbitrage, FixtureCluster, Game, MarketType, Outcome},
    services::{
        cluster_service::ClusterServiceErrors::ClusterNotFound,
        cluster_statistics::{ClusterStatistics, StatisticsValues},
        market_history_service::MarketHistoryService,
    },
};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct ClusterService {
    game_id_to_fixture_cluster_key: DashMap<String, String>,
    cluster_id_to_date: DashMap<String, NaiveDateTime>,
    clusters: DashMap<NaiveDateTime, DashMap<String, Arc<FixtureCluster>>>,
    event_tx: broadcast::Sender<Arc<FixtureCluster>>,
    statistics_event_tx: broadcast::Sender<Arc<StatisticsUpdated>>,
    market_history_service: Arc<MarketHistoryService>,
}

impl ClusterService {
    pub fn with_market_history_service(market_history_service: Arc<MarketHistoryService>) -> Self {
        ClusterService {
            clusters: DashMap::new(),
            game_id_to_fixture_cluster_key: DashMap::new(),
            cluster_id_to_date: DashMap::new(),
            event_tx: broadcast::Sender::new(20),
            statistics_event_tx: broadcast::Sender::new(20),
            market_history_service,
        }
    }

    fn emit_statistics_updated(&self) {
        let _ = self.statistics_event_tx.send(Arc::new(StatisticsUpdated {
            statistics: self.get_all_statistics(),
        }));
    }

    pub fn get_all_statistics(&self) -> HashMap<(MarketType, Outcome), StatisticsValues> {
        let mut aggregated: HashMap<(MarketType, Outcome), ClusterStatistics> = HashMap::new();

        for clusters_on_date in self.clusters.iter() {
            for cluster in clusters_on_date.value().iter() {
                for ((market_type, outcome), diff) in cluster.value().statistics_diffs() {
                    aggregated
                        .entry((market_type, outcome))
                        .or_default()
                        .add_diff(diff);
                }
            }
        }

        aggregated
            .into_iter()
            .map(|((market_type, outcome), stats)| {
                ((market_type, outcome), StatisticsValues::from(&stats))
            })
            .collect()
    }

    pub fn new() -> Self {
        ClusterService {
            clusters: DashMap::new(),
            game_id_to_fixture_cluster_key: DashMap::new(),
            cluster_id_to_date: DashMap::new(),
            event_tx: broadcast::Sender::new(20),
            statistics_event_tx: broadcast::Sender::new(20),
            market_history_service: Arc::new(MarketHistoryService::default()),
        }
    }

    fn add_games(&self, games: Vec<Game>) -> Vec<Arbitrage> {
        let mut arbitrages = Vec::new();
        for game in games {
            if self.game_id_to_fixture_cluster_key.contains_key(&game.id) {
                continue;
            }

            let mut found = false;
            let mut modified = false;
            let game_id = game.id.clone();
            let game_date = game.date.clone();
            let mut pending_game: Option<Game> = Some(game);

            if let Some(clusters) = self.clusters.get_mut(&game_date) {
                for mut cluster_ref in clusters.iter_mut() {
                    let cluster = cluster_ref.value_mut();
                    match Arc::make_mut(cluster).try_to_add_game(pending_game.take().unwrap()) {
                        Ok(_) => {
                            found = true;
                            modified = true;
                            let game_ref = cluster
                                .get_game(&game_id)
                                .expect("no game after try to add with success");
                            self.game_id_to_fixture_cluster_key
                                .entry(game_id.clone())
                                .or_insert(cluster.key());
                            self.cluster_id_to_date
                                .entry(cluster.key())
                                .insert_entry(game_date);
                            arbitrages.append(&mut cluster.arbitrage_opportunites());
                            self.market_history_service
                                .update_market_history(&game_id, game_ref.markets());

                            if cluster.game_count() > 1 {
                                let _ = self.event_tx.send(cluster.clone());
                            }

                            break;
                        }
                        Err(game) => pending_game = Some(game),
                    }
                }
            }

            if modified {
                self.emit_statistics_updated();
            }

            if !found {
                let game = pending_game.unwrap();

                let cluster = FixtureCluster::new(game);
                let cluster_key = cluster.key();
                self.clusters
                    .entry(game_date)
                    .or_insert_with(DashMap::new)
                    .entry(cluster_key.clone())
                    .or_insert(Arc::new(cluster));
                self.game_id_to_fixture_cluster_key
                    .entry(game_id)
                    .or_insert(cluster_key.clone());
                self.cluster_id_to_date
                    .entry(cluster_key)
                    .insert_entry(game_date);
                self.emit_statistics_updated();
            }
        }

        arbitrages
    }

    pub fn insert_games(&self, games: Vec<Game>) -> Vec<Arbitrage> {
        let mut arbitrages = Vec::new();

        games.into_iter().for_each(|game| {
            let game_id = &game.id;

            if let Some(cluster_id) = self.game_id_to_fixture_cluster_key.get(game_id) {
                self.clusters.entry(game.date).and_modify(|clusters_by_id| {
                    clusters_by_id
                        .entry(cluster_id.clone())
                        .and_modify(|cluster| {
                            if cluster.get_game(game_id).is_some() {
                                Arc::make_mut(cluster).update_markets(
                                    game_id,
                                    game.markets().values().cloned().collect(),
                                );

                                if cluster.game_count() > 1 {
                                    let _ = self.event_tx.send(cluster.clone());
                                }
                            }
                        });
                });
                self.emit_statistics_updated();
                if let Some(clusters_by_date) = self.clusters.get(&game.date)
                    && let Some(cluster) = clusters_by_date.value().get(cluster_id.as_str())
                {
                    arbitrages.append(&mut cluster.value().arbitrage_opportunites());
                }
                self.market_history_service
                    .update_market_history(&game_id, game.markets());
            } else {
                arbitrages.append(&mut self.add_games(vec![game]));
            }
        });

        arbitrages
    }

    pub fn insert_markets(&self, game_id: &str, markets: Vec<Market>) -> Vec<Arbitrage> {
        let mut modified = false;
        let mut arbitrages = Vec::new();

        if let Some(cluster_key_ref) = self.game_id_to_fixture_cluster_key.get(game_id)
            && let Some(game_date_ref) = self.cluster_id_to_date.get(cluster_key_ref.key())
            && let Some(games_on_date_ref) = self.clusters.get_mut(game_date_ref.value())
            && let Some(mut cluster_ref) = games_on_date_ref.value().get_mut(cluster_key_ref.key())
        {
            let cluster = cluster_ref.value_mut();
            if cluster.get_game(game_id).is_some() {
                modified = true;
                Arc::make_mut(cluster).update_markets(game_id, markets);

                let game = cluster
                    .get_game(game_id)
                    .expect("failed to get game in insert markets");
                let new_markets = game.markets();
                self.market_history_service
                    .update_market_history(&game_id, new_markets);

                arbitrages = cluster.arbitrage_opportunites();
            }
        }

        if modified {
            self.emit_statistics_updated();
        }

        arbitrages
    }

    pub fn get_clusters(&self) -> Vec<Arc<FixtureCluster>> {
        self.clusters
            .iter()
            .flat_map(|clusters_by_key_ref| {
                let clusters_by_key = clusters_by_key_ref.value();
                clusters_by_key
                    .iter()
                    .filter_map(|cluster_ref| {
                        let cluster = cluster_ref.value();
                        if cluster.game_count() > 1 {
                            Some(cluster.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Arc<FixtureCluster>>>()
            })
            .collect()
    }

    pub fn get_cluster(
        &self,
        cluster_id: &str,
    ) -> Result<Arc<FixtureCluster>, ClusterServiceErrors> {
        if let Some(cluster_date_ref) = self.cluster_id_to_date.get(cluster_id)
            && let Some(clusters_on_date_ref) = self.clusters.get(cluster_date_ref.value())
            && let Some(cluster_ref) = clusters_on_date_ref.value().get(cluster_id)
        {
            return Ok(cluster_ref.value().clone());
        }
        Err(ClusterNotFound)
    }

    pub fn subscribe_to_game_updates(&self) -> Receiver<Arc<FixtureCluster>> {
        self.event_tx.subscribe()
    }

    pub fn subscribe_to_cluster_statistics(&self) -> Receiver<Arc<StatisticsUpdated>> {
        self.statistics_event_tx.subscribe()
    }

    pub fn get_games(&self) -> Vec<Game> {
        self.clusters
            .iter()
            .flat_map(|clusters_ref| {
                clusters_ref
                    .value()
                    .iter()
                    .flat_map(|fixture_cluster_ref| {
                        fixture_cluster_ref
                            .value()
                            .games()
                            .cloned()
                            .collect::<Vec<Game>>()
                    })
                    .collect::<Vec<Game>>()
            })
            .collect()
    }

    pub fn get_plaftorm_games(&self, platform: &Platform) -> Vec<Game> {
        self.clusters
            .iter()
            .flat_map(|c| {
                c.value()
                    .iter()
                    .flat_map(|fixture_cluster| {
                        fixture_cluster
                            .value()
                            .platform_games(platform)
                            .cloned()
                            .collect::<Vec<Game>>()
                    })
                    .collect::<Vec<Game>>()
            })
            .collect()
    }

    pub fn market_history_service(&self) -> Arc<MarketHistoryService> {
        Arc::clone(&self.market_history_service)
    }
}

pub enum ClusterServiceErrors {
    ClusterNotFound,
}

#[derive(Debug, Clone)]
pub struct StatisticsUpdated {
    pub statistics: HashMap<(MarketType, Outcome), StatisticsValues>,
}

impl fmt::Display for ClusterService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for clusters_by_date_ref in self.clusters.iter() {
            let clusters_by_date = clusters_by_date_ref.value();
            for cluster_ref in clusters_by_date.iter() {
                writeln!(f, "{}", cluster_ref.value())?;
            }
        }

        Ok(())
    }
}

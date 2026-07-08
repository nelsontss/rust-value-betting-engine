use std::{
    fmt::{self},
    sync::Arc,
};

use chrono::NaiveDateTime;
use dashmap::DashMap;
use tokio::sync::broadcast::{self, Receiver};

use crate::domain::{
    Market, Platform,
    entities::{Arbitrage, FixtureCluster, Game},
    services::{
        cluster_service::ClusterServiceErrors::ClusterNotFound,
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
    market_history_service: Arc<MarketHistoryService>,
}

impl ClusterService {
    pub fn with_market_history_service(market_history_service: Arc<MarketHistoryService>) -> Self {
        ClusterService {
            clusters: DashMap::new(),
            game_id_to_fixture_cluster_key: DashMap::new(),
            cluster_id_to_date: DashMap::new(),
            event_tx: broadcast::Sender::new(20),
            market_history_service,
        }
    }

    pub fn new() -> Self {
        ClusterService {
            clusters: DashMap::new(),
            game_id_to_fixture_cluster_key: DashMap::new(),
            cluster_id_to_date: DashMap::new(),
            event_tx: broadcast::Sender::new(20),
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
            let game_id = game.id.clone();
            let game_date = game.date.clone();
            let mut pending_game: Option<Game> = Some(game);

            if let Some(clusters) = self.clusters.get_mut(&game_date) {
                for mut cluster_ref in clusters.iter_mut() {
                    let cluster = cluster_ref.value_mut();
                    match Arc::make_mut(cluster).try_to_add_game(pending_game.take().unwrap()) {
                        Ok(_) => {
                            found = true;
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

            if !found {
                let cluster = FixtureCluster::new(pending_game.unwrap());
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
                            Arc::make_mut(cluster).update_markets(
                                game_id,
                                game.markets().values().cloned().collect(),
                            );

                            if cluster.game_count() > 1 {
                                let _ = self.event_tx.send(cluster.clone());
                            }
                        });
                });
                arbitrages.append(
                    &mut self
                        .clusters
                        .get(&game.date)
                        .unwrap()
                        .value()
                        .get(cluster_id.as_str())
                        .unwrap() // TODO: I observed a panic here
                        .value()
                        .arbitrage_opportunites(),
                );
                self.market_history_service
                    .update_market_history(&game_id, game.markets());
            } else {
                arbitrages.append(&mut self.add_games(vec![game]));
            }
        });

        arbitrages
    }

    pub fn insert_markets(&self, game_id: &str, markets: Vec<Market>) -> Vec<Arbitrage> {
        if let Some(cluster_key_ref) = self.game_id_to_fixture_cluster_key.get(game_id)
            && let Some(game_date_ref) = self.cluster_id_to_date.get(cluster_key_ref.key())
            && let Some(games_on_date_ref) = self.clusters.get_mut(game_date_ref.value())
            && let Some(mut cluster_ref) = games_on_date_ref.value().get_mut(cluster_key_ref.key())
        {
            let cluster = cluster_ref.value_mut();
            Arc::make_mut(cluster).update_markets(game_id, markets);

            let game = cluster
                .get_game(game_id)
                .expect("failed to get game in insert markets");
            self.market_history_service
                .update_market_history(&game_id, game.markets());

            return cluster.arbitrage_opportunites();
        }

        vec![]
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

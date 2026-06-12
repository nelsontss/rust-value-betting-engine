use std::{
    collections::HashMap,
    fmt::{self},
    sync::Arc,
};

use chrono::NaiveDateTime;
use tokio::sync::broadcast::{self, Receiver};

use crate::domain::{
    Market, Platform,
    entities::{Arbitrage, FixtureCluster, Game},
    services::cluster_service::ClusterServiceErrors::ClusterNotFound,
};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct ClusterService {
    game_id_to_fixture_cluster_key: HashMap<String, String>,
    cluster_id_to_date: HashMap<String, NaiveDateTime>,
    clusters: HashMap<NaiveDateTime, HashMap<String, Arc<FixtureCluster>>>,
    event_tx: broadcast::Sender<Arc<FixtureCluster>>,
}

impl ClusterService {
    pub fn new() -> Self {
        ClusterService {
            clusters: HashMap::new(),
            game_id_to_fixture_cluster_key: HashMap::new(),
            cluster_id_to_date: HashMap::new(),
            event_tx: broadcast::Sender::new(20),
        }
    }

    fn add_games(&mut self, games: Vec<Game>) -> Vec<Arbitrage> {
        let mut arbitrages = Vec::new();
        for game in games {
            if self.game_id_to_fixture_cluster_key.contains_key(&game.id) {
                continue;
            }

            let mut found = false;
            let game_id = game.id.clone();
            let game_date = game.date.clone();
            let mut pending_game = Some(game);

            if let Some(clusters) = self.clusters.get_mut(&game_date) {
                for (_, cluster) in clusters.iter_mut() {
                    match Arc::make_mut(cluster).try_to_add_game(pending_game.take().unwrap()) {
                        Ok(_) => {
                            found = true;
                            self.game_id_to_fixture_cluster_key
                                .entry(game_id.clone())
                                .or_insert(cluster.key());
                            self.cluster_id_to_date
                                .entry(cluster.key())
                                .insert_entry(game_date);
                            arbitrages.append(&mut cluster.arbitrage_opportunites());

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
                    .or_insert_with(HashMap::new)
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

    pub fn insert_games(&mut self, games: Vec<Game>) -> Vec<Arbitrage> {
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
                        .get(cluster_id)
                        .unwrap()
                        .arbitrage_opportunites(),
                );
            } else {
                arbitrages.append(&mut self.add_games(vec![game]));
            }
        });

        arbitrages
    }

    pub fn insert_markets(&mut self, game_id: &str, markets: Vec<Market>) -> Vec<Arbitrage> {
        if let Some(cluster_key) = self.game_id_to_fixture_cluster_key.get(game_id)
            && let Some(game_date) = self.cluster_id_to_date.get(cluster_key)
            && let Some(games_on_date) = self.clusters.get_mut(game_date)
            && let Some(cluster) = games_on_date.get_mut(cluster_key)
        {
            Arc::make_mut(cluster).update_markets(game_id, markets);

            return cluster.arbitrage_opportunites();
        }

        vec![]
    }

    pub fn get_clusters(&self) -> Vec<Arc<FixtureCluster>> {
        self.clusters
            .values()
            .flat_map(|clusters_by_key| {
                clusters_by_key.values().filter_map(|cluster| {
                    if cluster.game_count() > 1 {
                        Some(cluster.clone())
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub fn get_cluster(
        &self,
        cluster_id: &str,
    ) -> Result<Arc<FixtureCluster>, ClusterServiceErrors> {
        if let Some(cluster_date) = self.cluster_id_to_date.get(cluster_id) {
            if let Some(clusters_on_date) = self.clusters.get(cluster_date) {
                if let Some(cluster) = clusters_on_date.get(cluster_id) {
                    return Ok(cluster.clone());
                }
            }
        }
        Err(ClusterNotFound)
    }

    pub fn subscribe_to_game_updates(&self) -> Receiver<Arc<FixtureCluster>> {
        self.event_tx.subscribe()
    }

    pub fn get_games(&self) -> impl Iterator<Item = &Game> {
        self.clusters.values().flat_map(|c| {
            c.values()
                .flat_map(|fixture_cluster| fixture_cluster.games())
        })
    }

    pub fn get_plaftorm_games(&self, platform: &Platform) -> impl Iterator<Item = &Game> {
        self.clusters.values().flat_map(|c| {
            c.values()
                .flat_map(|fixture_cluster| fixture_cluster.platform_games(platform))
        })
    }
}

pub enum ClusterServiceErrors {
    ClusterNotFound,
}

impl fmt::Display for ClusterService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (_, clusters_by_date) in &self.clusters {
            for (_, cluster) in clusters_by_date {
                writeln!(f, "{}", cluster)?;
            }
        }

        Ok(())
    }
}

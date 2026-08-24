use std::{
    collections::HashMap,
    fmt::{self},
    sync::Arc,
};

use chrono::{NaiveDateTime, Utc};
use dashmap::{DashMap, mapref::entry::Entry};
use tokio::sync::broadcast::{self, Receiver};

use crate::{
    domain::{
        Market, Platform,
        entities::{Arbitrage, FixtureCluster, Game, MarketType, Outcome},
        services::{
            cluster_service::ClusterServiceErrors::ClusterNotFound, market_service::MarketService,
            statistics_service::StatisticsService,
        },
    },
    infrastructure::repositories::fixture_cluster_repository::FixtureClusterRepository,
};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct ClusterService {
    game_id_to_fixture_cluster_key: DashMap<String, String>,
    cluster_id_to_date: DashMap<String, NaiveDateTime>,
    clusters: DashMap<NaiveDateTime, DashMap<String, Arc<FixtureCluster>>>,
    event_tx: broadcast::Sender<Arc<FixtureCluster>>,
    market_service: Option<Arc<MarketService>>,
    fixture_cluster_repository: Option<Arc<FixtureClusterRepository>>,
    statistics_service: Option<Arc<StatisticsService>>,
}

impl ClusterService {
    const SWEEP_INTERVAL_SECS: u64 = 5 * 60;
    const END_OF_GAME_GRACE_MINUTES: i64 = 100;

    pub fn new() -> Self {
        ClusterService {
            clusters: DashMap::new(),
            game_id_to_fixture_cluster_key: DashMap::new(),
            cluster_id_to_date: DashMap::new(),
            event_tx: broadcast::Sender::new(20),
            market_service: None,
            fixture_cluster_repository: None,
            statistics_service: None,
        }
    }

    pub fn with_market_service(mut self, market_service: Arc<MarketService>) -> Self {
        self.market_service = Some(market_service);

        self
    }

    pub fn with_fixture_cluster_repository(
        mut self,
        fixture_cluster_repository: Arc<FixtureClusterRepository>,
    ) -> Self {
        self.fixture_cluster_repository = Some(fixture_cluster_repository);

        self
    }

    pub fn with_statistics_service(mut self, statistics_service: Arc<StatisticsService>) -> Self {
        self.statistics_service = Some(statistics_service);

        self
    }

    pub fn persist_cluster(&self, cluster: Arc<FixtureCluster>) {
        if let Some(repo) = &self.fixture_cluster_repository {
            if tokio::runtime::Handle::try_current().is_err() {
                tracing::warn!(
                    cluster = cluster.key(),
                    "persist_cluster called outside a tokio runtime; skipping"
                );
                return;
            }

            let repo = Arc::clone(repo);
            tokio::spawn(async move {
                let key = cluster.key();
                if let Err(e) = repo.insert_cluster(&cluster).await {
                    tracing::error!("Error persist_cluster: {}: {}", key, e);
                }
            });
        }
    }

    pub fn persist_cluster_diffs(
        &self,
        key: String,
        mean_diffs: HashMap<(MarketType, Outcome), f64>,
    ) {
        if let Some(repo) = &self.fixture_cluster_repository {
            if tokio::runtime::Handle::try_current().is_err() {
                tracing::warn!(
                    cluster = key,
                    "persist_cluster_diffs called outside a tokio runtime; skipping"
                );
                return;
            }

            if mean_diffs.is_empty() {
                return;
            }

            let repo = Arc::clone(repo);
            tokio::spawn(async move {
                if let Err(e) = repo.insert_cluster_diffs(&key, &mean_diffs).await {
                    tracing::error!("Error persist_cluster_diffs: {}: {}", key, e);
                }
            });
        }
    }

    /// Completes the fixture lifecycle: persists the final state and the mean
    /// diffs, removes the cluster from memory and refreshes live statistics.
    fn end_cluster(&self, cluster_id: &str) {
        let Some(date_ref) = self.cluster_id_to_date.get(cluster_id) else {
            return;
        };
        let game_date = *date_ref.value();
        drop(date_ref);

        let mut ended: Option<Arc<FixtureCluster>> =
            self.clusters.get(&game_date).and_then(|clusters_on_date| {
                clusters_on_date
                    .value()
                    .remove(cluster_id)
                    .map(|(_, cluster)| cluster)
            });

        if let Some(cluster) = ended.as_mut() {
            Arc::make_mut(cluster).close();
        }

        if let Some(cluster) = ended.as_ref() {
            let mean_diffs = cluster.statistics_diffs();
            self.persist_cluster(Arc::clone(cluster));
            self.persist_cluster_diffs(cluster.key(), mean_diffs.clone());

            if let Some(stats_service) = &self.statistics_service {
                stats_service.add_completed_fixture_diffs(mean_diffs);
            }
        }

        self.game_id_to_fixture_cluster_key
            .retain(|_, v| v.as_str() != cluster_id);
        self.cluster_id_to_date.remove(cluster_id);

        if self
            .clusters
            .get(&game_date)
            .is_some_and(|clusters_on_date| clusters_on_date.is_empty())
        {
            self.clusters.remove(&game_date);
        }
    }

    /// Periodically ends clusters whose games started more than
    /// `END_OF_GAME_GRACE_MINUTES` ago.
    pub fn start_end_of_game_sweeper(self: &Arc<Self>) {
        let service = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(Self::SWEEP_INTERVAL_SECS));

            loop {
                interval.tick().await;
                service.sweep_ended_clusters();
            }
        });
    }

    fn sweep_ended_clusters(&self) {
        let cutoff =
            Utc::now().naive_utc() - chrono::Duration::minutes(Self::END_OF_GAME_GRACE_MINUTES);

        let elapsed_cluster_ids: Vec<String> = self
            .cluster_id_to_date
            .iter()
            .filter(|entry| *entry.value() < cutoff)
            .map(|entry| entry.key().clone())
            .collect();

        for cluster_id in elapsed_cluster_ids {
            self.end_cluster(&cluster_id);
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
            let game_date = game.date;
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
                            if let Some(market_service) = &self.market_service {
                                market_service.send_new_market_update(&game_id, game_ref.markets());
                            }

                            let _ = self.event_tx.send(cluster.clone());

                            self.persist_cluster(Arc::clone(cluster));

                            break;
                        }
                        Err(game) => pending_game = Some(game),
                    }
                }
            }

            if !found {
                let game = pending_game.unwrap();

                let cluster = Arc::new(FixtureCluster::new(game));
                let cluster_key = cluster.key();
                self.clusters
                    .entry(game_date)
                    .or_default()
                    .entry(cluster_key.clone())
                    .or_insert(cluster.clone());
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
                    if let Entry::Occupied(mut occupied) = clusters_by_id.entry(cluster_id.clone())
                    {
                        let cluster = occupied.get_mut();
                        if cluster.get_game(game_id).is_some() {
                            Arc::make_mut(cluster).update_markets(
                                game_id,
                                game.markets().values().cloned().collect(),
                            );

                            if cluster.game_count() > 1 {
                                let _ = self.event_tx.send(cluster.clone());

                                self.persist_cluster(Arc::clone(cluster));
                            }

                            arbitrages.append(&mut cluster.arbitrage_opportunites());
                            if let Some(market_service) = &self.market_service {
                                market_service.send_new_market_update(game_id, game.markets());
                            }
                        }
                    }
                });
            } else {
                arbitrages.append(&mut self.add_games(vec![game]));
            }
        });

        arbitrages
    }

    pub fn insert_markets(&self, game_id: &str, markets: Vec<Market>) -> Vec<Arbitrage> {
        let mut arbitrages = Vec::new();

        if let Some(cluster_key_ref) = self.game_id_to_fixture_cluster_key.get(game_id)
            && let Some(game_date_ref) = self.cluster_id_to_date.get(cluster_key_ref.key())
            && let Some(clusters_on_date_ref) = self.clusters.get_mut(game_date_ref.value())
            && let Some(mut cluster_ref) =
                clusters_on_date_ref.value().get_mut(cluster_key_ref.key())
        {
            let cluster = cluster_ref.value_mut();
            if cluster.get_game(game_id).is_some() {
                Arc::make_mut(cluster).update_markets(game_id, markets);

                let game = cluster
                    .get_game(game_id)
                    .expect("failed to get game in insert markets");
                let new_markets = game.markets();
                if let Some(market_service) = &self.market_service {
                    market_service.send_new_market_update(game_id, new_markets);
                }

                if cluster.game_count() > 1 {
                    self.persist_cluster(Arc::clone(&cluster));
                }

                arbitrages = cluster.arbitrage_opportunites();
            }
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

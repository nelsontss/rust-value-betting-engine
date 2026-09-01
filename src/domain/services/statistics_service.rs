use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::broadcast;

use crate::{
    domain::{
        entities::{MarketType, Outcome},
        services::cluster_statistics::{ClusterStatistics, StatisticsUpdated, StatisticsValues},
    },
    infrastructure::repositories::fixture_cluster_repository::FixtureClusterRepository,
    shared::error::Result,
};

#[derive(Debug)]
pub struct StatisticsService {
    fixture_cluster_repository: Arc<FixtureClusterRepository>,
    historical_diffs: DashMap<MarketType, HashMap<Outcome, ClusterStatistics>>,
    event_tx: broadcast::Sender<Arc<StatisticsUpdated>>,
}

impl StatisticsService {
    pub fn new(fixture_cluster_repository: Arc<FixtureClusterRepository>) -> Self {
        let (event_tx, _) = broadcast::channel(20);

        StatisticsService {
            fixture_cluster_repository,
            historical_diffs: DashMap::new(),
            event_tx,
        }
    }

    /// Rebuilds the historical distribution from persisted diffs.
    /// Called once at startup; afterwards updates are incremental only.
    pub async fn load_historical_diffs(&self) -> Result<()> {
        let diffs = self
            .fixture_cluster_repository
            .get_all_cluster_diffs()
            .await?;

        for (market_type, outcome, diff) in diffs {
            self.historical_diffs
                .entry(market_type)
                .or_default()
                .entry(outcome)
                .or_default()
                .add_diff(diff);
        }

        Ok(())
    }

    pub fn get_historical_statistics(
        &self,
    ) -> HashMap<MarketType, HashMap<Outcome, StatisticsValues>> {
        let mut out: HashMap<MarketType, HashMap<Outcome, StatisticsValues>> = HashMap::new();
        for entry in self.historical_diffs.iter() {
            let mut inner = HashMap::new();
            for (outcome, stats) in entry.value() {
                inner.insert(*outcome, StatisticsValues::from(stats));
            }
            out.insert(*entry.key(), inner);
        }
        out
    }

    pub fn add_completed_fixture_diffs(
        &self,
        mean_diffs: HashMap<MarketType, HashMap<Outcome, f64>>,
    ) {
        for (market_type, inner) in mean_diffs {
            for (outcome, diff) in inner {
                self.historical_diffs
                    .entry(market_type)
                    .or_default()
                    .entry(outcome)
                    .or_default()
                    .add_diff(diff);
            }
        }

        let statistics = self.get_historical_statistics();
        let _ = self
            .event_tx
            .send(Arc::new(StatisticsUpdated { statistics }));
    }

    pub fn subscribe_to_statistics(&self) -> broadcast::Receiver<Arc<StatisticsUpdated>> {
        self.event_tx.subscribe()
    }
}

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use tokio::sync::broadcast::{Receiver, Sender, error::RecvError};

use crate::domain::{
    ClusterService,
    entities::{FixtureCluster, MarketType, Outcome},
    services::{cluster_statistics::StatisticsValues, statistics_service::StatisticsService},
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct MarketClusterDiffDivergencyPayload {
    pub cluster_key: String,
    pub cluster_mean_diff: f64,
    pub statistics: StatisticsValues,
    pub market_type: MarketType,
    pub outcome: Outcome,
}

#[derive(Debug, Clone)]
pub struct AlertConvergencyPayload {
    pub cluster_key: String,
    pub cluster_mean_diff: f64,
    pub market_type: MarketType,
    pub outcome: Outcome,
    pub created_at: DateTime<Utc>,
    pub initial_polymarket_impl_prob: f64,
    pub current_polymarket_impl_prob: f64,
}

#[derive(Debug, Clone)]
pub enum AlertEvent {
    MarketClusterDiffDivergency(MarketClusterDiffDivergencyPayload),
    AlertConvergency(AlertConvergencyPayload),
}

#[derive(Debug, Clone)]
pub struct MonitorObject {
    pub market_type: MarketType,
    pub outcome: Outcome,
    pub end_date: DateTime<Utc>,
    pub polymarket_impl_prob: f64,
}

#[derive(Debug)]
struct AlertServiceInner {
    event_tx: Sender<Arc<AlertEvent>>,
    current_statistics: RwLock<HashMap<MarketType, HashMap<Outcome, StatisticsValues>>>,
    cluster_service: Arc<ClusterService>,
    statistics_service: Arc<StatisticsService>,
    clusters_to_monitor: DashMap<String, HashMap<MarketType, HashMap<Outcome, MonitorObject>>>,
}

impl AlertServiceInner {
    fn set_current_statistics(
        &self,
        new_statistics: HashMap<MarketType, HashMap<Outcome, StatisticsValues>>,
    ) {
        *self
            .current_statistics
            .write()
            .expect("current_statistics lock poisoned") = new_statistics;
    }

    fn check_monitors_for_regression(
        &self,
        cluster: &Arc<FixtureCluster>,
        cluster_current_diffs: &HashMap<MarketType, HashMap<Outcome, (f64, f64)>>,
        current_statistics: &HashMap<MarketType, HashMap<Outcome, StatisticsValues>>,
    ) {
        let Some(entry) = self.clusters_to_monitor.get(cluster.key().as_str()) else {
            return;
        };
        let snapshot: Vec<MonitorObject> = entry
            .values()
            .flat_map(|inner| inner.values().cloned())
            .collect();
        drop(entry);

        for mo in snapshot {
            let Some(market_diffs) = cluster_current_diffs.get(&mo.market_type) else {
                continue;
            };
            let Some(&(diff, diff_from_no)) = market_diffs.get(&mo.outcome) else {
                continue;
            };
            let Some(market_stats) = current_statistics.get(&mo.market_type) else {
                continue;
            };
            let Some(stats) = market_stats.get(&mo.outcome) else {
                continue;
            };
            self.monitor_regression_to_mean(cluster, &mo, diff, diff_from_no, stats)
        }
    }

    fn scan_for_divergency(
        &self,
        cluster: &Arc<FixtureCluster>,
        cluster_current_diffs: &HashMap<MarketType, HashMap<Outcome, (f64, f64)>>,
        current_statistics: &HashMap<MarketType, HashMap<Outcome, StatisticsValues>>,
    ) {
        for (market_type, inner) in cluster_current_diffs {
            for (outcome, &(diff, diff_from_no)) in inner {
                if diff == 0.0 && diff_from_no == 0.0 {
                    continue;
                }
                let Some(inner_stats) = current_statistics.get(market_type) else {
                    continue;
                };
                let Some(s) = inner_stats.get(outcome) else {
                    continue;
                };
                let Some(p05) = s.p05_diff else { continue };
                let Some(p95) = s.p95_diff else { continue };
                if diff >= p05 && diff_from_no <= p95 {
                    continue;
                }
                let _ = self
                    .event_tx
                    .send(Arc::new(AlertEvent::MarketClusterDiffDivergency(
                        MarketClusterDiffDivergencyPayload {
                            cluster_key: cluster.key(),
                            cluster_mean_diff: diff,
                            statistics: s.clone(),
                            market_type: *market_type,
                            outcome: *outcome,
                        },
                    )));

                let Some(polymarket_impl_prob) =
                    cluster.get_polymarket_impl_prob_of_market_and_outcome(market_type, outcome)
                else {
                    continue;
                };
                self.clusters_to_monitor
                    .entry(cluster.key())
                    .or_default()
                    .entry(*market_type)
                    .or_default()
                    .insert(
                        *outcome,
                        MonitorObject {
                            market_type: *market_type,
                            outcome: *outcome,
                            end_date: Utc::now() + Duration::minutes(5),
                            polymarket_impl_prob,
                        },
                    );
            }
        }
    }

    fn monitor_regression_to_mean(
        &self,
        cluster: &Arc<FixtureCluster>,
        mo: &MonitorObject,
        cluster_diff: f64,
        cluster_diff_from_no: f64,
        stats: &StatisticsValues,
    ) {
        let market_type = &mo.market_type;
        let outcome = &mo.outcome;

        if mo.end_date < Utc::now() {
            if let Some(mut outer) = self.clusters_to_monitor.get_mut(cluster.key().as_str()) {
                if let Some(inner) = outer.get_mut(market_type) {
                    inner.remove(outcome);
                    if inner.is_empty() {
                        outer.remove(market_type);
                    }
                }
                if outer.is_empty() {
                    drop(outer);
                    self.clusters_to_monitor.remove(cluster.key().as_str());
                }
            }
            return;
        }

        let Some(entry) = self.clusters_to_monitor.get(cluster.key().as_str()) else {
            return;
        };
        let Some(inner) = entry.get(market_type) else {
            return;
        };
        let Some(mo) = inner.get(outcome).cloned() else {
            return;
        };
        drop(entry);

        if let Some(impl_prob) =
            cluster.get_polymarket_impl_prob_of_market_and_outcome(market_type, outcome)
            && stats.p05_diff.is_some_and(|p05| cluster_diff >= p05)
            && stats
                .p95_diff
                .is_some_and(|p95| cluster_diff_from_no <= p95)
        {
            let _ = self.event_tx.send(Arc::new(AlertEvent::AlertConvergency(
                AlertConvergencyPayload {
                    cluster_key: cluster.key().to_string(),
                    cluster_mean_diff: cluster_diff,
                    market_type: *market_type,
                    outcome: *outcome,
                    created_at: Utc::now(),
                    initial_polymarket_impl_prob: mo.polymarket_impl_prob,
                    current_polymarket_impl_prob: impl_prob,
                },
            )));
        }
    }
}

pub struct AlertService {
    inner: Arc<AlertServiceInner>,
}

impl AlertService {
    pub fn new(
        cluster_service: Arc<ClusterService>,
        statistics_service: Arc<StatisticsService>,
    ) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(20);
        let inner_alert_service = Arc::new(AlertServiceInner {
            event_tx,
            current_statistics: RwLock::new(statistics_service.get_historical_statistics()),
            cluster_service,
            statistics_service,
            clusters_to_monitor: DashMap::new(),
        });
        let alert_service = AlertService {
            inner: inner_alert_service.clone(),
        };

        alert_service.handle_cluster_update_events();
        alert_service.handle_statistics_update_events();

        alert_service
    }

    pub fn subscribe_to_new_alerts(&self) -> Receiver<Arc<AlertEvent>> {
        self.inner.event_tx.subscribe()
    }

    fn handle_statistics_update_events(&self) {
        let inner_alert_service = self.inner.clone();

        tokio::spawn(async move {
            let mut rx = inner_alert_service
                .statistics_service
                .subscribe_to_statistics();

            loop {
                match rx.recv().await {
                    Ok(stats_updated) => {
                        inner_alert_service
                            .set_current_statistics(stats_updated.statistics.clone());
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        });
    }

    fn handle_cluster_update_events(&self) {
        let inner_alert_service = self.inner.clone();

        tokio::spawn(async move {
            let mut rx = inner_alert_service
                .cluster_service
                .subscribe_to_cluster_updates();
            loop {
                match rx.recv().await {
                    Ok(cluster_updated) => {
                        let cluster = cluster_updated.cluster;
                        let updated_markets = cluster_updated.updated_markets;
                        let cluster_current_diffs = cluster.live_statistics_diffs(&updated_markets);
                        let current_statistics = inner_alert_service
                            .current_statistics
                            .read()
                            .expect("current_statistics lock poisoned")
                            .clone();

                        inner_alert_service.check_monitors_for_regression(
                            &cluster,
                            &cluster_current_diffs,
                            &current_statistics,
                        );
                        inner_alert_service.scan_for_divergency(
                            &cluster,
                            &cluster_current_diffs,
                            &current_statistics,
                        );
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        });
    }
}

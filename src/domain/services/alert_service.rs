use std::{collections::HashMap, sync::Arc};

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
    pub cluster_key: String,
    pub market_type: MarketType,
    pub outcome: Outcome,
    pub end_date: DateTime<Utc>,
    pub polymarket_impl_prob: f64,
}

#[derive(Debug)]
pub struct AlertService {
    event_tx: Sender<Arc<AlertEvent>>,
}

impl AlertService {
    pub fn new(
        cluster_service: Arc<ClusterService>,
        statistics_service: Arc<StatisticsService>,
    ) -> Self {
        let cs = cluster_service.clone();
        let ss = statistics_service.clone();
        let (event_tx, _) = tokio::sync::broadcast::channel(20);
        let e_tx = event_tx.clone();
        let clusters_to_monitor: Arc<
            DashMap<String, HashMap<MarketType, HashMap<Outcome, MonitorObject>>>,
        > = Arc::new(DashMap::new());
        let monitors = clusters_to_monitor.clone();

        tokio::spawn(async move {
            let mut rx = cs.subscribe_to_cluster_updates();
            loop {
                match rx.recv().await {
                    Ok(c) => {
                        let cluster_current_diffs = c.live_statistics_diffs();
                        let current_statistics = ss.get_historical_statistics();

                        if let Some(entry) = monitors.get(&c.key()) {
                            let snapshot: Vec<MonitorObject> = entry
                                .values()
                                .flat_map(|inner| inner.values().cloned())
                                .collect();
                            drop(entry);
                            for mo in snapshot {
                                let Some(market_diffs) = cluster_current_diffs.get(&mo.market_type)
                                else {
                                    continue;
                                };
                                let Some(&(diff, diff_from_no)) = market_diffs.get(&mo.outcome)
                                else {
                                    continue;
                                };
                                let Some(market_stats) = current_statistics.get(&mo.market_type)
                                else {
                                    continue;
                                };
                                let Some(stats) = market_stats.get(&mo.outcome) else {
                                    continue;
                                };
                                monitor_regression_to_mean(
                                    c.clone(),
                                    mo.market_type,
                                    mo.outcome,
                                    diff,
                                    diff_from_no,
                                    stats,
                                    &mo.end_date,
                                    monitors.clone(),
                                    e_tx.clone(),
                                )
                            }
                        }

                        for (market_type, inner) in &cluster_current_diffs {
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
                                let _ =
                                    e_tx.send(Arc::new(AlertEvent::MarketClusterDiffDivergency(
                                        MarketClusterDiffDivergencyPayload {
                                            cluster_key: c.key(),
                                            cluster_mean_diff: diff,
                                            statistics: s.clone(),
                                            market_type: *market_type,
                                            outcome: *outcome,
                                        },
                                    )));

                                let Some(polymarket_impl_prob) = c
                                    .get_polymarket_impl_prob_of_market_and_outcome(
                                        market_type,
                                        outcome,
                                    )
                                else {
                                    continue;
                                };
                                monitors
                                    .entry(c.key())
                                    .or_default()
                                    .entry(*market_type)
                                    .or_default()
                                    .insert(
                                        *outcome,
                                        MonitorObject {
                                            cluster_key: c.key(),
                                            market_type: *market_type,
                                            outcome: *outcome,
                                            end_date: Utc::now() + Duration::minutes(5),
                                            polymarket_impl_prob,
                                        },
                                    );
                            }
                        }
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        });

        Self { event_tx }
    }

    pub fn subscribe_to_new_alerts(&self) -> Receiver<Arc<AlertEvent>> {
        self.event_tx.subscribe()
    }
}

fn monitor_regression_to_mean(
    cluster: Arc<FixtureCluster>,
    market_type: MarketType,
    outcome: Outcome,
    cluster_diff: f64,
    cluster_diff_from_no: f64,
    stats: &StatisticsValues,
    monitor_end_date: &DateTime<Utc>,
    monitors: Arc<DashMap<String, HashMap<MarketType, HashMap<Outcome, MonitorObject>>>>,
    event_tx: Sender<Arc<AlertEvent>>,
) {
    if *monitor_end_date < Utc::now() {
        if let Some(mut outer) = monitors.get_mut(&cluster.key()) {
            if let Some(inner) = outer.get_mut(&market_type) {
                inner.remove(&outcome);
                if inner.is_empty() {
                    outer.remove(&market_type);
                }
            }
            if outer.is_empty() {
                drop(outer);
                monitors.remove(&cluster.key());
            }
        }
        return;
    }

    let Some(entry) = monitors.get(&cluster.key()) else {
        return;
    };
    let Some(inner) = entry.get(&market_type) else {
        return;
    };
    let Some(mo) = inner.get(&outcome).cloned() else {
        return;
    };
    drop(entry);

    if let Some(impl_prob) =
        cluster.get_polymarket_impl_prob_of_market_and_outcome(&market_type, &outcome)
        && stats.p05_diff.is_some_and(|p05| cluster_diff >= p05)
        && stats.p95_diff.is_some_and(|p95| cluster_diff_from_no <= p95)
    {
        let _ = event_tx.send(Arc::new(AlertEvent::AlertConvergency(
            AlertConvergencyPayload {
                cluster_key: cluster.key().to_string(),
                cluster_mean_diff: cluster_diff,
                market_type,
                outcome,
                created_at: Utc::now(),
                initial_polymarket_impl_prob: mo.polymarket_impl_prob,
                current_polymarket_impl_prob: impl_prob,
            },
        )));
    }
}

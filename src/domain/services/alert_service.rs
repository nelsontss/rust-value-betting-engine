use std::sync::Arc;

use tokio::sync::broadcast::{Receiver, Sender, error::RecvError};

use crate::domain::{
    ClusterService,
    entities::{MarketType, Outcome},
    services::{cluster_statistics::StatisticsValues, statistics_service::StatisticsService},
};

#[derive(Debug, Clone)]
pub struct MarketClusterDiffDivergencyPayload {
    pub cluster_key: String,
    pub cluster_mean_diff: f64,
    pub statistics: StatisticsValues,
    pub market_type: MarketType,
    pub outcome: Outcome,
}

#[derive(Debug, Clone)]
pub enum AlertEvent {
    MarketClusterDiffDivergency(MarketClusterDiffDivergencyPayload),
}

#[derive(Debug)]
pub struct AlertService {
    cluster_service: Arc<ClusterService>,
    statistics_service: Arc<StatisticsService>,
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

        tokio::spawn(async move {
            let mut rx = cs.subscribe_to_cluster_updates();
            loop {
                match rx.recv().await {
                    Ok(c) => {
                        let cluster_current_diffs = c.live_statistics_diffs();

                        let current_statistics = ss.get_historical_statistics();

                        for (market_type, inner) in &cluster_current_diffs {
                            for (outcome, &diff) in inner {
                                if diff == 0.0 {
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
                                /*if s.samples < 20 {
                                    continue;
                                }*/
                                if diff >= p05 && diff <= p95 {
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
                            }
                        }
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        });

        Self {
            cluster_service,
            statistics_service,
            event_tx,
        }
    }

    pub fn subscribe_to_new_alerts(&self) -> Receiver<Arc<AlertEvent>> {
        self.event_tx.subscribe()
    }
}

use serde::Serialize;

use crate::domain::services::alert_service::{AlertEvent, MarketClusterDiffDivergencyPayload};

#[derive(Serialize)]
pub struct AlertResponse {
    pub r#type: String,
    pub payload: AlertPayloadResponse,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum AlertPayloadResponse {
    MarketClusterDiffDivergency(MarketClusterDiffDivergencyResponse),
}

#[derive(Serialize)]
pub struct MarketClusterDiffDivergencyResponse {
    pub cluster_key: String,
    pub cluster_mean_diff: f64,
    pub market_type: String,
    pub outcome: String,
    pub statistics: StatisticsValuesResponse,
}

#[derive(Serialize)]
pub struct StatisticsValuesResponse {
    pub samples: u64,
    pub mean_diff: f64,
    pub median_diff: Option<f64>,
    pub p05_diff: Option<f64>,
    pub p25_diff: Option<f64>,
    pub p75_diff: Option<f64>,
    pub p95_diff: Option<f64>,
}

impl From<&AlertEvent> for AlertResponse {
    fn from(event: &AlertEvent) -> Self {
        match event {
            AlertEvent::MarketClusterDiffDivergency(p) => AlertResponse {
                r#type: "MarketClusterDiffDivergency".to_string(),
                payload: AlertPayloadResponse::MarketClusterDiffDivergency(
                    MarketClusterDiffDivergencyResponse::from(p),
                ),
            },
        }
    }
}

impl From<&MarketClusterDiffDivergencyPayload> for MarketClusterDiffDivergencyResponse {
    fn from(p: &MarketClusterDiffDivergencyPayload) -> Self {
        Self {
            cluster_key: p.cluster_key.clone(),
            cluster_mean_diff: p.cluster_mean_diff,
            market_type: p.market_type.to_key_string(),
            outcome: format!("{:?}", p.outcome),
            statistics: StatisticsValuesResponse {
                samples: p.statistics.samples,
                mean_diff: p.statistics.mean_diff,
                median_diff: p.statistics.median_diff,
                p05_diff: p.statistics.p05_diff,
                p25_diff: p.statistics.p25_diff,
                p75_diff: p.statistics.p75_diff,
                p95_diff: p.statistics.p95_diff,
            },
        }
    }
}

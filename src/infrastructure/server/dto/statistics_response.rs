use std::collections::HashMap;

use serde::Serialize;

use crate::domain::services::cluster_statistics::{StatisticsUpdated, StatisticsValues};

#[derive(Serialize)]
pub struct StatisticsUpdatedResponse {
    pub statistics: HashMap<String, HashMap<String, StatisticsValuesResponse>>,
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

impl From<&StatisticsUpdated> for StatisticsUpdatedResponse {
    fn from(update: &StatisticsUpdated) -> Self {
        let mut statistics: HashMap<String, HashMap<String, StatisticsValuesResponse>> =
            HashMap::new();

        for (market_type, inner) in &update.statistics {
            for (outcome, values) in inner {
                statistics
                    .entry(market_type.to_key_string())
                    .or_default()
                    .insert(
                        format!("{:?}", outcome),
                        StatisticsValuesResponse::from(values),
                    );
            }
        }

        StatisticsUpdatedResponse { statistics }
    }
}

impl From<&StatisticsValues> for StatisticsValuesResponse {
    fn from(values: &StatisticsValues) -> Self {
        StatisticsValuesResponse {
            samples: values.samples,
            mean_diff: values.mean_diff,
            median_diff: values.median_diff,
            p05_diff: values.p05_diff,
            p25_diff: values.p25_diff,
            p75_diff: values.p75_diff,
            p95_diff: values.p95_diff,
        }
    }
}

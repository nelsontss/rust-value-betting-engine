use std::collections::HashMap;

use crate::domain::{
    entities::{MarketType, Outcome},
    services::quantile_multiset::QuantileMultiset,
};

/// Broadcast payload with the statistics snapshot per (market_type, outcome).
#[derive(Debug, Clone)]
pub struct StatisticsUpdated {
    pub statistics: HashMap<MarketType, HashMap<Outcome, StatisticsValues>>,
}

#[derive(Debug, Default, Clone)]
pub struct ClusterStatistics {
    sum_diff: f64,
    diff_values: QuantileMultiset,
}

#[derive(Debug, Clone)]
pub struct StatisticsValues {
    pub samples: u64,
    pub mean_diff: f64,
    pub median_diff: Option<f64>,
    pub p05_diff: Option<f64>,
    pub p25_diff: Option<f64>,
    pub p75_diff: Option<f64>,
    pub p95_diff: Option<f64>,
}

impl From<&ClusterStatistics> for StatisticsValues {
    fn from(stats: &ClusterStatistics) -> Self {
        StatisticsValues {
            samples: stats.samples(),
            mean_diff: stats.mean_diff(),
            median_diff: stats.median_diff(),
            p05_diff: stats.p05_diff(),
            p25_diff: stats.p25_diff(),
            p75_diff: stats.p75_diff(),
            p95_diff: stats.p95_diff(),
        }
    }
}

impl ClusterStatistics {
    pub(crate) fn add_diff(&mut self, value: f64) {
        self.sum_diff += value;
        self.diff_values.insert(value);
    }

    pub fn samples(&self) -> u64 {
        self.diff_values.len()
    }

    pub fn mean_diff(&self) -> f64 {
        if self.diff_values.is_empty() {
            0.0
        } else {
            self.sum_diff / self.diff_values.len() as f64
        }
    }

    pub fn median_diff(&self) -> Option<f64> {
        self.diff_values.median()
    }

    pub fn p05_diff(&self) -> Option<f64> {
        self.diff_values.p05()
    }

    pub fn p25_diff(&self) -> Option<f64> {
        self.diff_values.p25()
    }

    pub fn p75_diff(&self) -> Option<f64> {
        self.diff_values.p75()
    }

    pub fn p95_diff(&self) -> Option<f64> {
        self.diff_values.p95()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_diffs_mean_and_median() {
        let mut stats = ClusterStatistics::default();
        assert_eq!(0, stats.samples());
        assert_eq!(0.0, stats.mean_diff());
        assert_eq!(None, stats.median_diff());

        stats.add_diff(0.03);
        stats.add_diff(0.05);
        stats.add_diff(0.10);

        assert_eq!(3, stats.samples());
        assert!((stats.mean_diff() - 0.06).abs() < 1e-9);
        assert_eq!(Some(0.05), stats.median_diff());
    }
}

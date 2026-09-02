use std::collections::BTreeMap;

use ordered_float::OrderedFloat;

#[derive(Debug, Default, Clone)]
pub struct QuantileMultiset {
    values: BTreeMap<OrderedFloat<f64>, u64>,
    total: u64,
}

impl QuantileMultiset {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
            total: 0,
        }
    }

    pub fn insert(&mut self, value: f64) {
        *self.values.entry(OrderedFloat(value)).or_insert(0) += 1;
        self.total += 1;
    }

    pub fn remove(&mut self, value: f64) -> bool {
        if let Some(count) = self.values.get_mut(&OrderedFloat(value)) {
            *count -= 1;
            if *count == 0 {
                self.values.remove(&OrderedFloat(value));
            }
            self.total -= 1;
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> u64 {
        self.total
    }

    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    pub fn min(&self) -> Option<f64> {
        self.values.first_key_value().map(|(value, _)| value.0)
    }

    pub fn max(&self) -> Option<f64> {
        self.values.last_key_value().map(|(value, _)| value.0)
    }

    pub fn median(&self) -> Option<f64> {
        self.quantile(0.5)
    }

    pub fn p05(&self) -> Option<f64> {
        self.quantile(0.05)
    }

    pub fn p25(&self) -> Option<f64> {
        self.quantile(0.25)
    }

    pub fn p75(&self) -> Option<f64> {
        self.quantile(0.75)
    }

    pub fn p95(&self) -> Option<f64> {
        self.quantile(0.95)
    }

    pub fn quantile(&self, q: f64) -> Option<f64> {
        if self.total == 0 {
            return None;
        }

        let h = q * (self.total as f64 - 1.0);
        let lower_idx = h.floor() as u64;
        let upper_idx = h.ceil() as u64;
        let weight = h - lower_idx as f64;

        let lower = self.select(lower_idx)?;
        let upper = self.select(upper_idx)?;

        Some(lower + (upper - lower) * weight)
    }

    fn select(&self, rank: u64) -> Option<f64> {
        let mut remaining = rank + 1;
        for (value, count) in self.values.iter() {
            if *count >= remaining {
                return Some(value.0);
            }
            remaining -= count;
        }
        None
    }
}

#[cfg(test)]
mod tests;

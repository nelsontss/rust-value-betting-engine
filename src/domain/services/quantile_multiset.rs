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
mod tests {
    use super::*;

    #[test]
    fn median_of_sorted_distinct_values() {
        let mut multiset = QuantileMultiset::new();
        for value in [-1.2, -0.8, 0.3, 0.9, 1.5, 2.1, 3.4] {
            multiset.insert(value);
        }

        assert_eq!(Some(0.9), multiset.median());
    }

    #[test]
    fn median_of_even_count_averages_middle_two() {
        let mut multiset = QuantileMultiset::new();
        for value in [1.0, 2.0, 3.0, 4.0] {
            multiset.insert(value);
        }

        assert_eq!(Some(2.5), multiset.median());
    }

    #[test]
    fn quantile_interpolates_between_adjacent_values() {
        let mut multiset = QuantileMultiset::new();
        for value in [1.0, 2.0, 3.0, 4.0] {
            multiset.insert(value);
        }

        // p25: h = 0.25 * 3 = 0.75 -> between rank 0 (1.0) and rank 1 (2.0), weight 0.75
        assert_eq!(Some(1.75), multiset.p25());
        // p75: h = 0.75 * 3 = 2.25 -> between rank 2 (3.0) and rank 3 (4.0), weight 0.25
        assert_eq!(Some(3.25), multiset.p75());
    }

    #[test]
    fn insert_duplicate_values_are_counted() {
        let mut multiset = QuantileMultiset::new();
        multiset.insert(2.0);
        multiset.insert(2.0);
        multiset.insert(2.0);
        multiset.insert(4.0);

        assert_eq!(4, multiset.len());
        assert_eq!(Some(2.0), multiset.median());
    }

    #[test]
    fn remove_decrements_count_and_drops_key_at_zero() {
        let mut multiset = QuantileMultiset::new();
        multiset.insert(1.0);
        multiset.insert(2.0);
        multiset.insert(2.0);

        multiset.remove(2.0);
        assert_eq!(2, multiset.len());
        multiset.remove(2.0);
        assert_eq!(1, multiset.len());

        multiset.remove(99.0);
        assert_eq!(1, multiset.len());
    }

    #[test]
    fn replace_is_remove_then_insert() {
        let mut multiset = QuantileMultiset::new();
        multiset.insert(-0.8);
        multiset.insert(0.3);
        multiset.insert(0.9);

        assert_eq!(Some(0.3), multiset.median());

        multiset.remove(-0.8);
        multiset.insert(4.0);

        assert_eq!(Some(0.9), multiset.median());
    }

    #[test]
    fn empty_multiset_returns_none() {
        let multiset = QuantileMultiset::new();
        assert_eq!(None, multiset.median());
        assert_eq!(None, multiset.quantile(0.25));
        assert!(multiset.is_empty());
    }
}

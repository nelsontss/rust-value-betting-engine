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

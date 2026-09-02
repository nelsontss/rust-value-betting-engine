use super::*;
use polymarket_client_sdk_v2::types::dec;

#[test]
fn odd_rejects_initialization_with_non_positive_doubles() {
    assert!(matches!(Odd::new(-1.0), Err(OddError::NonPositive(v)) if v == -1.0));
    assert!(matches!(Odd::new(0.0), Err(OddError::NonPositive(v)) if v == 0.0));
    assert!(matches!(Odd::new(f64::NAN), Err(OddError::NonPositive(v)) if v.is_nan()));
}

#[test]
fn odd_rejects_initialization_with_invalid_probabilities() {
    assert_eq!(
        OddError::InvalidProbability(Decimal::ZERO),
        Odd::new_from_prob(Decimal::ZERO, Decimal::ONE).unwrap_err()
    );
    assert_eq!(
        OddError::InvalidProbability(dec!(1.5)),
        Odd::new_from_prob(dec!(1.5), dec!(0.5)).unwrap_err()
    );
}

#[test]
fn odd_round_trips_between_decimal_and_probability() {
    let odd = Odd::new(2.5).unwrap();
    let prob = odd.get_implied_probability();
    let reconstructed = Odd::new_from_prob(prob, Decimal::ONE - prob).unwrap();
    assert_eq!(odd.odd, reconstructed.odd);
}

#[test]
fn odd_new_stores_odd_and_implied_probability() {
    let odd = Odd::new(4.0).unwrap();

    assert_eq!(4.0, odd.get());
    assert_eq!(Decimal::from_f64(0.25).unwrap(), odd.get_implied_probability());
    assert_eq!(None, odd.get_implied_probability_derived_from_no());
}

#[test]
fn odd_new_accepts_very_small_and_very_large_finite_values() {
    assert!(Odd::new(1_000_000.0).is_ok());
    assert!(Odd::new(1.01).is_ok());
}

#[test]
fn odd_new_rejects_infinity() {
    assert!(matches!(Odd::new(f64::INFINITY), Err(OddError::NonPositive(_))));
}

#[test]
fn odd_new_from_prob_computes_odd_and_tracks_probability_derived_from_no() {
    let odd = Odd::new_from_prob(dec!(0.4), dec!(0.42)).unwrap();

    assert!((odd.get() - 2.5).abs() < 1e-9);
    assert_eq!(dec!(0.4), odd.get_implied_probability());
    assert_eq!(Some(dec!(0.58)), odd.get_implied_probability_derived_from_no());
}

#[test]
fn best_odd_with_id_returns_highest_odd_and_its_id() {
    let best = best_odd_with_id(vec![
        (Odd::new(1.9).unwrap(), "book-a".to_string()),
        (Odd::new(2.3).unwrap(), "book-b".to_string()),
        (Odd::new(2.1).unwrap(), "book-c".to_string()),
    ]);

    assert!((best.0.get() - 2.3).abs() < 1e-12);
    assert_eq!("book-b", best.1);
}

#[test]
fn best_odd_with_id_breaks_ties_deterministically() {
    let best = best_odd_with_id(vec![
        (Odd::new(2.0).unwrap(), "book-a".to_string()),
        (Odd::new(2.0).unwrap(), "book-b".to_string()),
    ]);

    assert_eq!(2.0, best.0.get());
    assert!(best.1 == "book-a" || best.1 == "book-b");
}

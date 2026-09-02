use super::{Arbitrage, MatchResultArbitrage, TwoWayArbitrage};
use crate::domain::entities::Odd;

fn assert_close(expected: f64, actual: f64) {
    assert!(
        (expected - actual).abs() < 0.000_001,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn stake_distribution_equalizes_payouts_for_two_way_arbitrage() {
    let arbitrage = Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
        (Odd::new(2.2).unwrap(), "home-book".to_string()),
        (Odd::new(2.2).unwrap(), "away-book".to_string()),
        (1.0 / 2.2) + (1.0 / 2.2),
    ));

    let distribution = arbitrage.stake_distribution(100.0).unwrap();

    assert_eq!(2, distribution.stakes.len());
    assert_close(100.0, distribution.total_stake);
    assert_close(110.0, distribution.guaranteed_payout);
    assert_close(10.0, distribution.guaranteed_profit);
    assert_close(0.1, distribution.roi);
    assert_close(50.0, distribution.stakes[0].stake);
    assert_close(50.0, distribution.stakes[1].stake);
    assert_close(
        distribution.guaranteed_payout,
        distribution.stakes[0].payout,
    );
    assert_close(
        distribution.guaranteed_payout,
        distribution.stakes[1].payout,
    );
}

#[test]
fn stake_distribution_supports_three_way_arbitrage() {
    let arbitrage = Arbitrage::MatchResultArbitrage(MatchResultArbitrage::new(
        (Odd::new(2.2).unwrap(), "home-book".to_string()),
        (Odd::new(3.6).unwrap(), "draw-book".to_string()),
        (Odd::new(4.1).unwrap(), "away-book".to_string()),
        (1.0 / 2.2) + (1.0 / 3.6) + (1.0 / 4.1),
    ));

    let distribution = arbitrage.stake_distribution(100.0).unwrap();

    assert_eq!(
        vec!["home", "draw", "away"],
        distribution
            .stakes
            .iter()
            .map(|stake| stake.outcome)
            .collect::<Vec<_>>()
    );
    assert!(distribution.guaranteed_profit > 0.0);
    assert_close(
        distribution.guaranteed_payout,
        distribution.stakes[0].payout,
    );
    assert_close(
        distribution.guaranteed_payout,
        distribution.stakes[1].payout,
    );
    assert_close(
        distribution.guaranteed_payout,
        distribution.stakes[2].payout,
    );
}

#[test]
fn stake_distribution_rejects_invalid_total_stakes() {
    let arbitrage = Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
        (Odd::new(2.2).unwrap(), "home-book".to_string()),
        (Odd::new(2.2).unwrap(), "away-book".to_string()),
        (1.0 / 2.2) + (1.0 / 2.2),
    ));

    assert_eq!(None, arbitrage.stake_distribution(0.0));
    assert_eq!(None, arbitrage.stake_distribution(-10.0));
    assert_eq!(None, arbitrage.stake_distribution(f64::NAN));
}

#[test]
fn implied_probability_sum_and_roi_match_for_each_variant() {
    let two_way = Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
        (Odd::new(2.2).unwrap(), "a".to_string()),
        (Odd::new(2.2).unwrap(), "b".to_string()),
        0.9090909090909091,
    ));
    assert_close(0.9090909090909091, two_way.implied_probability_sum());
    assert_close(0.1, two_way.roi());

    let two_way_line = Arbitrage::TwoWayLineArbitrage(super::TwoWayLineArbitrage::new(
        crate::domain::entities::markets::Line(-0.5),
        (Odd::new(2.2).unwrap(), "a".to_string()),
        (Odd::new(2.2).unwrap(), "b".to_string()),
        0.9090909090909091,
    ));
    assert_close(0.1, two_way_line.roi());

    let three_way_line = Arbitrage::ThreeWayLineArbitrage(super::ThreeWayLineArbitrage::new(
        crate::domain::entities::markets::Line(-1.0),
        (Odd::new(4.2).unwrap(), "a".to_string()),
        (Odd::new(4.2).unwrap(), "b".to_string()),
        (Odd::new(2.4).unwrap(), "c".to_string()),
        0.8928571428571429,
    ));
    assert_close(0.12, three_way_line.roi(), );
}

#[test]
fn guaranteed_payout_and_profit_derive_from_implied_probability_sum() {
    let arbitrage = Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
        (Odd::new(2.2).unwrap(), "a".to_string()),
        (Odd::new(2.2).unwrap(), "b".to_string()),
        (1.0 / 2.2) + (1.0 / 2.2),
    ));

    let payout = arbitrage.guaranteed_payout(100.0).unwrap();
    let profit = arbitrage.guaranteed_profit(100.0).unwrap();

    assert_close(110.0, payout);
    assert_close(10.0, profit);
    assert_close(profit, payout - 100.0);
}

#[test]
fn guaranteed_payout_rejects_non_positive_and_non_finite_stakes() {
    let arbitrage = Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
        (Odd::new(2.2).unwrap(), "a".to_string()),
        (Odd::new(2.2).unwrap(), "b".to_string()),
        (1.0 / 2.2) + (1.0 / 2.2),
    ));

    assert_eq!(None, arbitrage.guaranteed_payout(0.0));
    assert_eq!(None, arbitrage.guaranteed_payout(-5.0));
    assert_eq!(None, arbitrage.guaranteed_payout(f64::NAN));
    assert_eq!(None, arbitrage.guaranteed_payout(f64::INFINITY));
}

#[test]
fn stake_distribution_for_line_arbitrage_equalizes_payouts() {
    let arbitrage = Arbitrage::TwoWayLineArbitrage(super::TwoWayLineArbitrage::new(
        crate::domain::entities::markets::Line(-0.5),
        (Odd::new(2.2).unwrap(), "a".to_string()),
        (Odd::new(2.2).unwrap(), "b".to_string()),
        (1.0 / 2.2) + (1.0 / 2.2),
    ));

    let distribution = arbitrage.stake_distribution(200.0).unwrap();

    assert_eq!(vec!["home", "away"], distribution.stakes.iter().map(|s| s.outcome).collect::<Vec<_>>());
    assert_close(100.0, distribution.stakes[0].stake);
    assert_close(100.0, distribution.stakes[1].stake);
    assert_close(220.0, distribution.guaranteed_payout);
    assert_close(20.0, distribution.guaranteed_profit);
}

#[test]
fn stake_recommendation_payout_equals_stake_times_odd() {
    let arbitrage = Arbitrage::MatchResultArbitrage(MatchResultArbitrage::new(
        (Odd::new(2.2).unwrap(), "a".to_string()),
        (Odd::new(3.6).unwrap(), "b".to_string()),
        (Odd::new(4.1).unwrap(), "c".to_string()),
        (1.0 / 2.2) + (1.0 / 3.6) + (1.0 / 4.1),
    ));

    let distribution = arbitrage.stake_distribution(300.0).unwrap();

    for stake in &distribution.stakes {
        assert_close(stake.stake * stake.odd.get(), stake.payout);
    }
    assert_close(300.0, distribution.stakes.iter().map(|s| s.stake).sum());
}

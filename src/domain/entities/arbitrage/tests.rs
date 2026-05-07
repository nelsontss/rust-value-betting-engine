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

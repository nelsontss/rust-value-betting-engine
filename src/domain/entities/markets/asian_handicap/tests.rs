use super::*;

#[test]
fn asian_handicap_market_arbitrage_opportunites_handles_quarter_lines() {
    let first_market = AsianHandicapMarket::new(
        "first-asian".to_string(),
        Line(-0.25),
        Odd::new(2.2).unwrap(),
        Odd::new(1.8).unwrap(),
    );
    let second_market = AsianHandicapMarket::new(
        "second-asian".to_string(),
        Line(-0.25),
        Odd::new(1.8).unwrap(),
        Odd::new(2.2).unwrap(),
    );

    let result = AsianHandicapMarket::arbitrage_opportunites(&vec![first_market, second_market]);

    assert!(matches!(result, Some(Arbitrage::TwoWayLineArbitrage(_))));
}

#[test]
fn asian_handicap_arbitrage_opportunites_returns_none_for_empty_markets() {
    let result = AsianHandicapMarket::arbitrage_opportunites(&vec![]);

    assert_eq!(None, result);
}

#[test]
fn asian_handicap_arbitrage_opportunites_returns_none_when_lines_differ() {
    let first = AsianHandicapMarket::new(
        "first-asian".to_string(),
        Line(-0.5),
        Odd::new(2.2).unwrap(),
        Odd::new(1.8).unwrap(),
    );
    let second = AsianHandicapMarket::new(
        "second-asian".to_string(),
        Line(-1.0),
        Odd::new(1.8).unwrap(),
        Odd::new(2.2).unwrap(),
    );

    let result = AsianHandicapMarket::arbitrage_opportunites(&vec![first, second]);

    assert_eq!(None, result);
}

#[test]
fn asian_handicap_arbitrage_opportunites_finds_arbitrage_on_half_lines() {
    let first = AsianHandicapMarket::new(
        "first-asian".to_string(),
        Line(-0.5),
        Odd::new(2.2).unwrap(),
        Odd::new(1.8).unwrap(),
    );
    let second = AsianHandicapMarket::new(
        "second-asian".to_string(),
        Line(-0.5),
        Odd::new(1.8).unwrap(),
        Odd::new(2.2).unwrap(),
    );

    let result = AsianHandicapMarket::arbitrage_opportunites(&vec![first, second]);

    match result {
        Some(arb @ Arbitrage::TwoWayLineArbitrage(_)) => {
            assert!((arb.implied_probability_sum() - 0.9090909090909091).abs() < 1e-9);
            assert!(arb.stake_distribution(100.0).unwrap().guaranteed_profit > 0.0);
        }
        other => panic!("expected two way line arbitrage, got {:?}", other),
    }
}

#[test]
fn asian_handicap_id_returns_stored_id() {
    let market = AsianHandicapMarket::new(
        "ah-id".to_string(),
        Line(-0.5),
        Odd::new(2.0).unwrap(),
        Odd::new(2.0).unwrap(),
    );

    assert_eq!("ah-id", market.id());
}

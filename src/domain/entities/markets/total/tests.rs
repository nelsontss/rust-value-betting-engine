use super::*;

#[test]
fn total_market_arbitrage_opportunites_rejects_integer_lines_with_push_state() {
    let first_market = TotalMarket::new(
        "first-total".to_string(),
        Line(2.0),
        Odd::new(2.2).unwrap(),
        Odd::new(1.8).unwrap(),
    );
    let second_market = TotalMarket::new(
        "second-total".to_string(),
        Line(2.0),
        Odd::new(1.8).unwrap(),
        Odd::new(2.2).unwrap(),
    );

    let result = TotalMarket::arbitrage_opportunites(&vec![first_market, second_market]);

    assert_eq!(None, result);
}

#[test]
fn total_market_arbitrage_opportunites_returns_none_for_empty_markets() {
    let result = TotalMarket::arbitrage_opportunites(&vec![]);

    assert_eq!(None, result);
}

#[test]
fn total_market_arbitrage_opportunites_returns_none_when_lines_differ() {
    let first = TotalMarket::new(
        "first-total".to_string(),
        Line(2.5),
        Odd::new(2.2).unwrap(),
        Odd::new(1.8).unwrap(),
    );
    let second = TotalMarket::new(
        "second-total".to_string(),
        Line(3.5),
        Odd::new(1.8).unwrap(),
        Odd::new(2.2).unwrap(),
    );

    let result = TotalMarket::arbitrage_opportunites(&vec![first, second]);

    assert_eq!(None, result);
}

#[test]
fn total_market_arbitrage_opportunites_finds_cross_bookmaker_arbitrage_on_half_lines() {
    let first = TotalMarket::new(
        "first-total".to_string(),
        Line(2.5),
        Odd::new(2.2).unwrap(),
        Odd::new(1.8).unwrap(),
    );
    let second = TotalMarket::new(
        "second-total".to_string(),
        Line(2.5),
        Odd::new(1.8).unwrap(),
        Odd::new(2.2).unwrap(),
    );

    let result = TotalMarket::arbitrage_opportunites(&vec![first, second]);

    match result {
        Some(arb @ Arbitrage::TwoWayLineArbitrage(_)) => {
            assert!((arb.implied_probability_sum() - 0.9090909090909091).abs() < 1e-9);
            assert!(arb.stake_distribution(100.0).unwrap().guaranteed_profit > 0.0);
        }
        other => panic!("expected two way line arbitrage, got {:?}", other),
    }
}

#[test]
fn total_market_id_returns_stored_id() {
    let market = TotalMarket::new(
        "total-id".to_string(),
        Line(2.5),
        Odd::new(2.0).unwrap(),
        Odd::new(2.0).unwrap(),
    );

    assert_eq!("total-id", market.id());
}

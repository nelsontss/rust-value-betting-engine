use super::*;

#[test]
fn handicap_arbitrage_returns_none_for_empty_markets() {
    let result = HandicapMarket::arbitrage_opportunites(&vec![]);

    assert_eq!(None, result);
}

#[test]
fn handicap_arbitrage_returns_none_when_lines_differ() {
    let first = HandicapMarket::new(
        "first",
        Line(-1.0),
        Odd::new(3.0).unwrap(),
        Odd::new(3.5).unwrap(),
        Odd::new(2.2).unwrap(),
    );
    let second = HandicapMarket::new(
        "second",
        Line(-0.5),
        Odd::new(3.0).unwrap(),
        Odd::new(3.5).unwrap(),
        Odd::new(2.2).unwrap(),
    );

    let result = HandicapMarket::arbitrage_opportunites(&vec![first, second]);

    assert_eq!(None, result);
}

#[test]
fn handicap_arbitrage_returns_none_when_no_arbitrage_exists() {
    let market = HandicapMarket::new(
        "single",
        Line(-1.0),
        Odd::new(2.0).unwrap(),
        Odd::new(3.0).unwrap(),
        Odd::new(2.0).unwrap(),
    );

    let result = HandicapMarket::arbitrage_opportunites(&vec![market]);

    assert_eq!(None, result);
}

#[test]
fn handicap_arbitrage_returns_three_way_line_arbitrage_when_profitable() {
    let first = HandicapMarket::new(
        "first",
        Line(-1.0),
        Odd::new(4.2).unwrap(),
        Odd::new(3.6).unwrap(),
        Odd::new(2.0).unwrap(),
    );
    let second = HandicapMarket::new(
        "second",
        Line(-1.0),
        Odd::new(3.5).unwrap(),
        Odd::new(4.2).unwrap(),
        Odd::new(2.0).unwrap(),
    );
    let third = HandicapMarket::new(
        "third",
        Line(-1.0),
        Odd::new(3.5).unwrap(),
        Odd::new(3.5).unwrap(),
        Odd::new(2.4).unwrap(),
    );

    let result = HandicapMarket::arbitrage_opportunites(&vec![first, second, third]);

    match result {
        Some(arb @ Arbitrage::ThreeWayLineArbitrage(_)) => {
            assert!(arb.implied_probability_sum() < 1.0);
            assert!(arb.stake_distribution(100.0).unwrap().guaranteed_profit > 0.0);
        }
        other => panic!("expected three way line arbitrage, got {:?}", other),
    }
}

#[test]
fn handicap_id_returns_stored_id() {
    let market = HandicapMarket::new(
        "hc-id",
        Line(0.0),
        Odd::new(2.0).unwrap(),
        Odd::new(3.0).unwrap(),
        Odd::new(2.0).unwrap(),
    );

    assert_eq!("hc-id", market.id());
}

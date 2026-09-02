use super::*;

#[test]
fn double_chance_arbitrage_best_pair_found() {
    let market = DoubleChanceMarket::new(
        "single".to_string(),
        Odd::new(2.2).unwrap(),
        Odd::new(2.2).unwrap(),
        Odd::new(2.2).unwrap(),
    );

    let result = DoubleChanceMarket::arbitrage_opportunites(&vec![market]);

    assert!(matches!(result, Some(Arbitrage::TwoWayArbitrage(_))));
}

#[test]
fn double_chance_arbitrage_picks_best_pair() {
    let first = DoubleChanceMarket::new(
        "first".to_string(),
        Odd::new(2.0).unwrap(),
        Odd::new(3.0).unwrap(),
        Odd::new(100.0).unwrap(),
    );
    let second = DoubleChanceMarket::new(
        "second".to_string(),
        Odd::new(2.4).unwrap(),
        Odd::new(2.0).unwrap(),
        Odd::new(2.0).unwrap(),
    );

    let result = DoubleChanceMarket::arbitrage_opportunites(&vec![first, second]).unwrap();

    // best pair should be 1X+12: 1/2.4 + 1/3 = 0.75 -> roi = 33.3%
    // 1X+X2: 1/2.4 + 1/100 = 0.427 -> roi = 134%
    // 12+X2: 1/3 + 1/100 = 0.343 -> roi = 191%
    // So 12+X2 has best roi at 191%
    assert!((result.roi() - 1.913).abs() < 0.01);
}

#[test]
fn double_chance_arbitrage_returns_none_when_no_arb() {
    let market = DoubleChanceMarket::new(
        "single".to_string(),
        Odd::new(1.3).unwrap(),
        Odd::new(1.4).unwrap(),
        Odd::new(1.5).unwrap(),
    );

    let result = DoubleChanceMarket::arbitrage_opportunites(&vec![market]);

    assert_eq!(None, result);
}

#[test]
fn double_chance_arbitrage_returns_none_for_empty_markets() {
    let result = DoubleChanceMarket::arbitrage_opportunites(&vec![]);

    assert_eq!(None, result);
}

use super::{Arbitrage, Line, MarketGroup, Odd, TotalMarket};

#[test]
fn market_group_total_arbitrage_accepts_lines_with_same_canonical_key() {
    let first_market = TotalMarket::new(
        "first-total".to_string(),
        Line(2.5),
        Odd::new(2.15).unwrap(),
        Odd::new(1.75).unwrap(),
    );
    let second_market = TotalMarket::new(
        "second-total".to_string(),
        Line(2.5000002),
        Odd::new(1.8).unwrap(),
        Odd::new(2.15).unwrap(),
    );

    let group = MarketGroup::Total {
        line: 250,
        markets: vec![first_market, second_market],
    };

    let result = group.arbitrage();

    assert!(matches!(result, Some(Arbitrage::TwoWayLineArbitrage(_))));
}

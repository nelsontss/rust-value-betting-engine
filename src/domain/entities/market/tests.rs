use crate::domain::entities::market::OddError;

use super::{Arbitrage, AsianHandicapMarket, Line, MarketGroup, Odd, TotalMarket};

#[test]
fn odd_rejects_initialization_with_non_positive_doubles() {
    assert_eq!(OddError::NonPositive(-1.0), Odd::new(-1.0).unwrap_err());
    assert_eq!(OddError::NonPositive(0.0), Odd::new(0.0).unwrap_err());
}
#[test]
fn market_group_total_arbitrage_accepts_lines_with_same_canonical_key() {
    let first_market = TotalMarket {
        id: "first-total".to_string(),
        line: Line(2.5),
        over: Odd(2.15),
        under: Odd(1.75),
    };
    let second_market = TotalMarket {
        id: "second-total".to_string(),
        line: Line(2.5000002),
        over: Odd(1.8),
        under: Odd(2.15),
    };

    let group = MarketGroup::Total {
        line: 250,
        markets: vec![&first_market, &second_market],
    };

    let result = group.arbitrage();

    assert!(matches!(result, Some(Arbitrage::TwoWayLineArbitrage(_))));
}

#[test]
fn total_market_arbitrage_opportunites_rejects_integer_lines_with_push_state() {
    let first_market = TotalMarket {
        id: "first-total".to_string(),
        line: Line(2.0),
        over: Odd::new(2.2).unwrap(),
        under: Odd::new(1.8).unwrap(),
    };
    let second_market = TotalMarket {
        id: "second-total".to_string(),
        line: Line(2.0),
        over: Odd::new(1.8).unwrap(),
        under: Odd::new(2.2).unwrap(),
    };

    let result = TotalMarket::arbitrage_opportunites(&[&first_market, &second_market]);

    assert_eq!(None, result);
}

#[test]
fn asian_handicap_market_arbitrage_opportunites_handles_quarter_lines() {
    let first_market = AsianHandicapMarket {
        id: "first-asian".to_string(),
        line: Line(-0.25),
        home: Odd::new(2.2).unwrap(),
        away: Odd::new(1.8).unwrap(),
    };
    let second_market = AsianHandicapMarket {
        id: "second-asian".to_string(),
        line: Line(-0.25),
        home: Odd::new(1.8).unwrap(),
        away: Odd::new(2.2).unwrap(),
    };

    let result = AsianHandicapMarket::arbitrage_opportunites(&[&first_market, &second_market]);

    assert!(matches!(result, Some(Arbitrage::TwoWayLineArbitrage(_))));
}

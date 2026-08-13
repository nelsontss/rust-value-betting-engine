use super::{Arbitrage, Line, Market, MarketGroup, MarketType, Odd, Outcome, TotalMarket};

#[test]
fn odd_for_outcome_returns_odd_for_valid_outcome() {
    let market = Market::match_result("id", 2.0, 3.0, 4.0).unwrap();

    assert_eq!(2.0, market.odd_for_outcome(&Outcome::Home).unwrap().get());
    assert_eq!(3.0, market.odd_for_outcome(&Outcome::Draw).unwrap().get());
    assert_eq!(4.0, market.odd_for_outcome(&Outcome::Away).unwrap().get());
}

#[test]
fn odd_for_outcome_returns_none_for_outcome_of_another_market_type() {
    let market = Market::match_result("id", 2.0, 3.0, 4.0).unwrap();

    assert_eq!(None, market.odd_for_outcome(&Outcome::Over));
    assert_eq!(None, market.odd_for_outcome(&Outcome::HomeOrDraw));
}

#[test]
fn double_chance_exposes_three_combination_outcomes() {
    let market = Market::double_chance("id", 1.2, 1.5, 1.8).unwrap();

    assert_eq!(
        1.2,
        market.odd_for_outcome(&Outcome::HomeOrDraw).unwrap().get()
    );
    assert_eq!(
        1.5,
        market.odd_for_outcome(&Outcome::HomeOrAway).unwrap().get()
    );
    assert_eq!(
        1.8,
        market.odd_for_outcome(&Outcome::DrawOrAway).unwrap().get()
    );
}

#[test]
fn every_market_type_outcome_has_an_odd() {
    let markets = vec![
        Market::match_result("mr", 2.0, 3.0, 4.0).unwrap(),
        Market::moneyline("ml", 1.8, 2.0).unwrap(),
        Market::double_chance("dc", 1.2, 1.5, 1.8).unwrap(),
        Market::total("tl", 2.5, 1.9, 1.9).unwrap(),
        Market::handicap("hc", 0.0, 2.2, 3.4, 3.2).unwrap(),
        Market::asian_handicap("ah", -0.5, 2.0, 1.8).unwrap(),
    ];

    for market in markets {
        let market_type = MarketType::from(&market);
        for outcome in market_type.outcomes() {
            assert!(
                market.odd_for_outcome(&outcome).is_some(),
                "missing odd for {:?} {:?}",
                market_type,
                outcome
            );
        }
    }
}

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

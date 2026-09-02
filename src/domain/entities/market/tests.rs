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

#[test]
fn market_type_key_round_trips_for_every_variant() {
    let types = vec![
        MarketType::MatchResult,
        MarketType::Moneyline,
        MarketType::DoubleChance,
        MarketType::Total { line: 250 },
        MarketType::Handicap { line: -100 },
        MarketType::AsianHandicap { line: -25 },
    ];

    for market_type in types {
        let key = market_type.to_key_string();
        assert_eq!(Some(market_type), MarketType::from_key_string(&key));
    }
}

#[test]
fn market_type_from_key_string_rejects_malformed_keys() {
    assert_eq!(None, MarketType::from_key_string("Total"));
    assert_eq!(None, MarketType::from_key_string("Total:not-a-number"));
    assert_eq!(None, MarketType::from_key_string("MatchResult:250"));
    assert_eq!(None, MarketType::from_key_string("Unknown"));
    assert_eq!(None, MarketType::from_key_string(""));
}

#[test]
fn market_type_variant_name_matches_variant() {
    assert_eq!("MatchResult", MarketType::MatchResult.variant_name());
    assert_eq!("Moneyline", MarketType::Moneyline.variant_name());
    assert_eq!("DoubleChance", MarketType::DoubleChance.variant_name());
    assert_eq!("Total", MarketType::Total { line: 1 }.variant_name());
    assert_eq!("Handicap", MarketType::Handicap { line: 1 }.variant_name());
    assert_eq!(
        "AsianHandicap",
        MarketType::AsianHandicap { line: 1 }.variant_name()
    );
}

#[test]
fn outcome_from_key_string_parses_all_variants() {
    assert_eq!(Some(Outcome::Home), Outcome::from_key_string("Home"));
    assert_eq!(Some(Outcome::Draw), Outcome::from_key_string("Draw"));
    assert_eq!(Some(Outcome::Away), Outcome::from_key_string("Away"));
    assert_eq!(Some(Outcome::Over), Outcome::from_key_string("Over"));
    assert_eq!(Some(Outcome::Under), Outcome::from_key_string("Under"));
    assert_eq!(Some(Outcome::HomeOrDraw), Outcome::from_key_string("HomeOrDraw"));
    assert_eq!(Some(Outcome::HomeOrAway), Outcome::from_key_string("HomeOrAway"));
    assert_eq!(Some(Outcome::DrawOrAway), Outcome::from_key_string("DrawOrAway"));
    assert_eq!(None, Outcome::from_key_string("Nope"));
}

#[test]
fn sum_implied_probabilities_for_fair_two_way_market_is_one() {
    let market = Market::moneyline("ml", 2.0, 2.0).unwrap();

    assert!((market.sum_implied_probabilities() - 1.0).abs() < 1e-9);
}

#[test]
fn sum_implied_probabilities_for_bookmaker_market_exceeds_one() {
    let market = Market::match_result("mr", 2.0, 3.0, 4.0).unwrap();

    // 0.5 + 0.3333 + 0.25 = 1.0833
    assert!((market.sum_implied_probabilities() - 1.0833333333333333).abs() < 1e-9);
}

#[test]
fn market_group_from_market_exposes_market_type_and_accepts_same_type() {
    let mut group = MarketGroup::from_market(moneyline());
    assert_eq!(MarketType::Moneyline, group.market_type());
    assert!(group.push_market(moneyline()).is_ok());

    let mut group = MarketGroup::from_market(total(250, 2.0));
    assert_eq!(MarketType::Total { line: 250 }, group.market_type());
    assert!(group.push_market(total(250, 2.2)).is_ok());

    let mut group = MarketGroup::from_market(asian_handicap(-25));
    assert_eq!(MarketType::AsianHandicap { line: -25 }, group.market_type());
    assert!(group.push_market(asian_handicap(-25)).is_ok());

    let mut group = MarketGroup::from_market(handicap(-100));
    assert_eq!(MarketType::Handicap { line: -100 }, group.market_type());
    assert!(group.push_market(handicap(-100)).is_ok());

    let mut group = MarketGroup::from_market(match_result());
    assert_eq!(MarketType::MatchResult, group.market_type());
    assert!(group.push_market(match_result()).is_ok());

    let mut group = MarketGroup::from_market(double_chance());
    assert_eq!(MarketType::DoubleChance, group.market_type());
    assert!(group.push_market(double_chance()).is_ok());
}

#[test]
fn market_group_push_market_rejects_different_types() {
    let mut group = MarketGroup::from_market(moneyline());

    assert!(matches!(
        group.push_market(match_result()),
        Err(super::MarketGroupError::MarketTypeAndGroupDontMatch)
    ));
}

#[test]
fn market_group_push_market_rejects_different_lines() {
    let mut group = MarketGroup::from_market(total(250, 2.0));

    assert!(group.push_market(total(300, 2.2)).is_err());
}

#[test]
fn market_group_arbitrage_delegates_to_moneyline_detector() {
    let mut group = MarketGroup::from_market(Market::moneyline("a", 2.2, 1.8).unwrap());
    assert!(group.push_market(Market::moneyline("b", 1.8, 2.2).unwrap()).is_ok());

    let result = group.arbitrage();

    assert!(matches!(result, Some(Arbitrage::TwoWayArbitrage(_))));
}

#[test]
fn market_constructors_propagate_odd_errors() {
    assert!(Market::match_result("id", 0.0, 3.0, 4.0).is_err());
    assert!(Market::moneyline("id", -1.0, 2.0).is_err());
    assert!(Market::double_chance("id", 1.2, 0.0, 1.8).is_err());
    assert!(Market::total("id", 2.5, 0.0, 1.9).is_err());
    assert!(Market::handicap("id", -1.0, 2.0, 3.0, -2.0).is_err());
    assert!(Market::asian_handicap("id", -0.5, f64::NAN, 2.0).is_err());
}

fn moneyline() -> Market {
    Market::moneyline("ml", 2.0, 1.8).unwrap()
}

fn total(line: i32, over_odd: f64) -> Market {
    Market::total("total", line as f32 / 100.0, over_odd, 1.9).unwrap()
}

fn asian_handicap(line: i32) -> Market {
    Market::asian_handicap("ah", line as f32 / 100.0, 2.0, 1.8).unwrap()
}

fn handicap(line: i32) -> Market {
    Market::handicap("hc", line as f32 / 100.0, 2.0, 3.0, 1.8).unwrap()
}

fn match_result() -> Market {
    Market::match_result("mr", 2.0, 3.0, 4.0).unwrap()
}

fn double_chance() -> Market {
    Market::double_chance("dc", 1.2, 1.5, 1.8).unwrap()
}

#[test]
fn odd_for_outcome_returns_none_for_mismatches_on_every_variant() {
    let cases = vec![
        (Market::moneyline("ml", 2.0, 2.0).unwrap(), Outcome::Draw),
        (Market::total("total", 2.5, 2.0, 2.0).unwrap(), Outcome::Home),
        (Market::handicap("hc", -1.0, 2.0, 3.0, 2.0).unwrap(), Outcome::Over),
        (
            Market::asian_handicap("ah", -0.5, 2.0, 2.0).unwrap(),
            Outcome::Draw,
        ),
        (
            Market::double_chance("dc", 1.2, 1.5, 1.8).unwrap(),
            Outcome::Away,
        ),
    ];

    for (market, outcome) in cases {
        assert_eq!(None, market.odd_for_outcome(&outcome));
    }
}

#[test]
fn market_group_push_market_accepts_moneyline_and_double_chance_variants() {
    let mut moneyline_group = MarketGroup::from_market(Market::moneyline("ml", 2.0, 1.8).unwrap());
    assert!(moneyline_group
        .push_market(Market::moneyline("ml2", 2.1, 1.7).unwrap())
        .is_ok());

    let mut double_chance_group =
        MarketGroup::from_market(Market::double_chance("dc", 1.2, 1.5, 1.8).unwrap());
    assert!(double_chance_group
        .push_market(Market::double_chance("dc2", 1.3, 1.6, 1.7).unwrap())
        .is_ok());
}

#[test]
fn market_group_push_market_accepts_handicap_and_asian_handicap_lines() {
    let mut handicap_group = MarketGroup::from_market(Market::handicap("hc", -1.0, 2.0, 3.0, 1.8).unwrap());
    assert!(handicap_group
        .push_market(Market::handicap("hc2", -1.0, 2.1, 3.1, 1.7).unwrap())
        .is_ok());

    let mut asian_group =
        MarketGroup::from_market(Market::asian_handicap("ah", -0.5, 2.0, 1.8).unwrap());
    assert!(asian_group
        .push_market(Market::asian_handicap("ah2", -0.5, 2.1, 1.7).unwrap())
        .is_ok());
}

#[test]
fn market_group_arbitrage_delegates_for_every_variant() {
    // single markets with no arbitrage must yield None on every group type
    let groups = vec![
        MarketGroup::from_market(match_result()),
        MarketGroup::from_market(moneyline()),
        MarketGroup::from_market(double_chance()),
        MarketGroup::from_market(total(250, 2.0)),
        MarketGroup::from_market(handicap(-100)),
        MarketGroup::from_market(asian_handicap(-25)),
    ];

    for group in groups {
        assert_eq!(None, group.arbitrage());
    }
}

#[test]
fn market_group_arbitrage_finds_cross_book_profit_for_every_variant() {
    // match result: best odds across three books sum below one
    let mut match_result_group = MarketGroup::from_market(Market::match_result("a", 2.6, 3.0, 3.0).unwrap());
    match_result_group.push_market(Market::match_result("b", 2.0, 3.6, 3.0).unwrap()).ok();
    match_result_group.push_market(Market::match_result("c", 2.0, 3.0, 3.7).unwrap()).ok();
    assert!(match_result_group.arbitrage().is_some());

    let mut double_chance_group = MarketGroup::from_market(Market::double_chance("a", 2.2, 2.2, 2.2).unwrap());
    double_chance_group.push_market(Market::double_chance("b", 2.2, 2.2, 2.2).unwrap()).ok();
    assert!(double_chance_group.arbitrage().is_some());

    let mut total_group = MarketGroup::from_market(Market::total("a", 2.5, 2.2, 1.8).unwrap());
    total_group.push_market(Market::total("b", 2.5, 1.8, 2.2).unwrap()).ok();
    assert!(total_group.arbitrage().is_some());

    let mut handicap_group = MarketGroup::from_market(Market::handicap("a", -1.0, 4.2, 3.6, 2.0).unwrap());
    handicap_group.push_market(Market::handicap("b", -1.0, 3.5, 4.2, 2.0).unwrap()).ok();
    handicap_group.push_market(Market::handicap("c", -1.0, 3.5, 3.5, 2.4).unwrap()).ok();
    assert!(handicap_group.arbitrage().is_some());

    let mut asian_group = MarketGroup::from_market(Market::asian_handicap("a", -0.5, 2.2, 1.8).unwrap());
    asian_group.push_market(Market::asian_handicap("b", -0.5, 1.8, 2.2).unwrap()).ok();
    assert!(asian_group.arbitrage().is_some());
}

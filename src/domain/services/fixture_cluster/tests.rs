use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::domain::{
    entities::{
        Arbitrage, Game, Line, Market, MarketGroup, MarketType, MoneylineMarket, Odd, TotalMarket,
    },
    services::FixtureCluster,
};

fn fixture_date(hour: u32, min: u32) -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
        NaiveTime::from_hms_milli_opt(hour, min, 0, 0).unwrap(),
    )
}

fn game_with_markets(platform: &str, markets: Vec<(MarketType, Market)>) -> Game {
    let mut game = Game::new(
        "FC Porto",
        "SL Benfica",
        "Portugal",
        "Liga Portugal",
        fixture_date(15, 30),
        platform,
    );

    game.markets.extend(markets);

    game
}

fn moneyline_market(id: &str, home: f64, away: f64) -> (MarketType, Market) {
    (
        MarketType::Moneyline,
        Market::Moneyline(MoneylineMarket::new(
            id,
            Odd::new(home).unwrap(),
            Odd::new(away).unwrap(),
        )),
    )
}

fn total_market(id: &str, line: f32, over: f64, under: f64) -> (MarketType, Market) {
    (
        MarketType::Total {
            line: (line * 100.0) as i32,
        },
        Market::Total(TotalMarket::new(
            id,
            Line(line),
            Odd::new(over).unwrap(),
            Odd::new(under).unwrap(),
        )),
    )
}

fn total_market_type(line: f32) -> MarketType {
    MarketType::Total {
        line: (line * 100.0) as i32,
    }
}

fn as_moneyline(market: &Market) -> &MoneylineMarket {
    match market {
        Market::Moneyline(market) => market,
        _ => panic!("expected moneyline market"),
    }
}

fn as_total(market: &Market) -> &TotalMarket {
    match market {
        Market::Total(market) => market,
        _ => panic!("expected total market"),
    }
}

fn assert_moneyline_group<'a>(cluster: &'a FixtureCluster, expected: Vec<&'a MoneylineMarket>) {
    match cluster.markets.get(&MarketType::Moneyline) {
        Some(MarketGroup::Moneyline(markets)) => assert_eq!(&expected, markets),
        Some(_) => panic!("expected moneyline market group"),
        None => panic!("expected moneyline market group to exist"),
    }
}

fn assert_total_group<'a>(
    cluster: &'a FixtureCluster,
    market_type: &MarketType,
    expected: Vec<&'a TotalMarket>,
) {
    match cluster.markets.get(market_type) {
        Some(MarketGroup::Total { markets, .. }) => assert_eq!(&expected, markets),
        Some(_) => panic!("expected total market group"),
        None => panic!("expected total market group to exist"),
    }
}

#[test]
fn new_groups_initial_game_markets_by_type() {
    let game = game_with_markets(
        "Betano",
        vec![
            moneyline_market("betano-moneyline", 2.0, 1.8),
            total_market("betano-total", 2.5, 1.9, 1.9),
        ],
    );

    let cluster = FixtureCluster::new(&game);

    assert_eq!(1, cluster.game_count());
    assert_moneyline_group(
        &cluster,
        vec![as_moneyline(&game.markets[&MarketType::Moneyline])],
    );
    assert_total_group(
        &cluster,
        &total_market_type(2.5),
        vec![as_total(&game.markets[&total_market_type(2.5)])],
    );
}

#[test]
fn try_to_add_game_appends_markets_to_existing_groups() {
    let first_game = game_with_markets(
        "Betano",
        vec![
            moneyline_market("betano-moneyline", 2.0, 1.8),
            total_market("betano-total", 2.5, 1.9, 1.9),
        ],
    );
    let second_game = game_with_markets(
        "Betclic",
        vec![
            moneyline_market("betclic-moneyline", 2.1, 1.75),
            total_market("betclic-total", 2.5, 2.0, 1.85),
        ],
    );

    let mut cluster = FixtureCluster::new(&first_game);

    assert!(cluster.try_to_add_game(&second_game));

    assert_eq!(2, cluster.game_count());
    assert_moneyline_group(
        &cluster,
        vec![
            as_moneyline(&first_game.markets[&MarketType::Moneyline]),
            as_moneyline(&second_game.markets[&MarketType::Moneyline]),
        ],
    );
    assert_total_group(
        &cluster,
        &total_market_type(2.5),
        vec![
            as_total(&first_game.markets[&total_market_type(2.5)]),
            as_total(&second_game.markets[&total_market_type(2.5)]),
        ],
    );
}

#[test]
fn try_to_add_game_creates_a_group_for_new_market_types() {
    let first_game = game_with_markets(
        "Betano",
        vec![moneyline_market("betano-moneyline", 2.0, 1.8)],
    );
    let second_game = game_with_markets(
        "Betclic",
        vec![
            moneyline_market("betclic-moneyline", 2.1, 1.75),
            total_market("betclic-total", 2.5, 2.0, 1.85),
        ],
    );

    let mut cluster = FixtureCluster::new(&first_game);

    assert!(cluster.try_to_add_game(&second_game));

    assert_eq!(2, cluster.game_count());
    assert_moneyline_group(
        &cluster,
        vec![
            as_moneyline(&first_game.markets[&MarketType::Moneyline]),
            as_moneyline(&second_game.markets[&MarketType::Moneyline]),
        ],
    );
    assert_total_group(
        &cluster,
        &total_market_type(2.5),
        vec![as_total(&second_game.markets[&total_market_type(2.5)])],
    );
}

#[test]
fn try_to_add_game_keeps_total_markets_with_different_lines_in_separate_groups() {
    let first_game = game_with_markets("Betano", vec![total_market("betano-total", 2.5, 1.9, 1.9)]);
    let second_game = game_with_markets(
        "Betclic",
        vec![total_market("betclic-total", 3.0, 2.0, 1.85)],
    );

    let mut cluster = FixtureCluster::new(&first_game);

    assert!(cluster.try_to_add_game(&second_game));

    assert_eq!(2, cluster.game_count());
    assert_eq!(2, cluster.markets.len());
    assert_total_group(
        &cluster,
        &total_market_type(2.5),
        vec![as_total(&first_game.markets[&total_market_type(2.5)])],
    );
    assert_total_group(
        &cluster,
        &total_market_type(3.0),
        vec![as_total(&second_game.markets[&total_market_type(3.0)])],
    );
}

#[test]
fn arbitrage_opportunites_returns_multiple_arbitrage_types_together() {
    let first_game = game_with_markets(
        "Betano",
        vec![
            moneyline_market("betano-moneyline", 2.2, 1.7),
            total_market("betano-total", 2.5, 2.15, 1.75),
        ],
    );
    let second_game = game_with_markets(
        "Betclic",
        vec![
            moneyline_market("betclic-moneyline", 1.8, 2.2),
            total_market("betclic-total", 2.5, 1.8, 2.15),
        ],
    );

    let mut cluster = FixtureCluster::new(&first_game);

    assert!(cluster.try_to_add_game(&second_game));

    let arbitrage_opportunities = cluster.arbitrage_opportunites();

    assert_eq!(2, arbitrage_opportunities.len());
    assert!(
        arbitrage_opportunities
            .iter()
            .any(|opportunity| matches!(opportunity, Arbitrage::TwoWayArbitrage(_)))
    );
    assert!(
        arbitrage_opportunities
            .iter()
            .any(|opportunity| matches!(opportunity, Arbitrage::TwoWayLineArbitrage(_)))
    );
}

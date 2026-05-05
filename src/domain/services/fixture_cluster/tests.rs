use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::domain::{
    entities::{Game, Line, Market, MarketType, MoneylineMarket, Odd, TotalMarket},
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
        Market::Moneyline(MoneylineMarket::new(id, Odd(home), Odd(away))),
    )
}

fn total_market(id: &str, line: f64, over: f64, under: f64) -> (MarketType, Market) {
    (
        MarketType::Total {
            line: line.to_string(),
        },
        Market::Total(TotalMarket::new(id, Line(line), Odd(over), Odd(under))),
    )
}

fn total_market_type(line: f64) -> MarketType {
    MarketType::Total {
        line: line.to_string(),
    }
}

fn assert_market_group(cluster: &FixtureCluster, market_type: &MarketType, expected: Vec<&Market>) {
    assert_eq!(Some(&expected), cluster.markets.get(market_type));
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
    assert_market_group(
        &cluster,
        &MarketType::Moneyline,
        vec![&game.markets[&MarketType::Moneyline]],
    );
    assert_market_group(
        &cluster,
        &total_market_type(2.5),
        vec![&game.markets[&total_market_type(2.5)]],
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
    assert_market_group(
        &cluster,
        &MarketType::Moneyline,
        vec![
            &first_game.markets[&MarketType::Moneyline],
            &second_game.markets[&MarketType::Moneyline],
        ],
    );
    assert_market_group(
        &cluster,
        &total_market_type(2.5),
        vec![
            &first_game.markets[&total_market_type(2.5)],
            &second_game.markets[&total_market_type(2.5)],
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
    assert_market_group(
        &cluster,
        &MarketType::Moneyline,
        vec![
            &first_game.markets[&MarketType::Moneyline],
            &second_game.markets[&MarketType::Moneyline],
        ],
    );
    assert_market_group(
        &cluster,
        &total_market_type(2.5),
        vec![&second_game.markets[&total_market_type(2.5)]],
    );
}

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::domain::entities::{
    Arbitrage, FixtureCluster, Game, Market, MarketGroup, MarketType, Odd, Platform,
    markets::{Line, moneyline::MoneylineMarket, total::TotalMarket},
};

fn fixture_date(hour: u32, min: u32) -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
        NaiveTime::from_hms_milli_opt(hour, min, 0, 0).unwrap(),
    )
}

fn game_with_markets(platform: Platform, markets: Vec<(MarketType, Market)>) -> Game {
    Game::new(
        "FC Porto",
        "SL Benfica",
        "Portugal",
        "Liga Portugal",
        fixture_date(15, 30),
        platform,
        markets.into_iter().map(|(_, market)| market).collect(),
    )
}

fn moneyline_market(id: &str, home: f64, away: f64) -> (MarketType, Market) {
    (
        MarketType::Moneyline,
        Market::Moneyline(MoneylineMarket::new(
            id.to_string(),
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
            id.to_string(),
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

fn as_moneyline(market: &Market) -> MoneylineMarket {
    match market {
        Market::Moneyline(market) => market.clone(),
        _ => panic!("expected moneyline market"),
    }
}

fn as_total(market: &Market) -> TotalMarket {
    match market {
        Market::Total(market) => market.clone(),
        _ => panic!("expected total market"),
    }
}

fn assert_moneyline_group(cluster: &FixtureCluster, mut expected: Vec<MoneylineMarket>) {
    let market_type = MarketType::Moneyline;
    let game_ids = cluster.market_type_to_game_ids.get(&market_type).unwrap();

    match cluster.build_market_group((&market_type, game_ids)) {
        Some(MarketGroup::Moneyline(mut markets)) => {
            expected.sort_by_key(|market| market.id().to_string());
            markets.sort_by_key(|market| market.id().to_string());
            assert_eq!(expected, markets)
        }
        Some(_) => panic!("expected moneyline market group"),
        None => panic!("expected moneyline market group to exist"),
    }
}

fn assert_total_group(
    cluster: &FixtureCluster,
    market_type: &MarketType,
    mut expected: Vec<TotalMarket>,
) {
    let game_ids = cluster.market_type_to_game_ids.get(market_type).unwrap();

    match cluster.build_market_group((&market_type, game_ids)) {
        Some(MarketGroup::Total { mut markets, .. }) => {
            expected.sort_by_key(|market| {
                (
                    market.line.key(),
                    market.over.get().to_bits(),
                    market.under.get().to_bits(),
                )
            });
            markets.sort_by_key(|market| {
                (
                    market.line.key(),
                    market.over.get().to_bits(),
                    market.under.get().to_bits(),
                )
            });
            assert_eq!(expected, markets)
        }
        Some(_) => panic!("expected total market group"),
        None => panic!("expected total market group to exist"),
    }
}

#[test]
fn new_groups_initial_game_markets_by_type() {
    let game = game_with_markets(
        Platform::Betano,
        vec![
            moneyline_market("betano-moneyline", 2.0, 1.8),
            total_market("betano-total", 2.5, 1.9, 1.9),
        ],
    );

    let cluster = FixtureCluster::new(game.clone());

    assert_eq!(1, cluster.game_count());
    assert_moneyline_group(
        &cluster,
        vec![as_moneyline(&game.markets()[&MarketType::Moneyline])],
    );
    assert_total_group(
        &cluster,
        &total_market_type(2.5),
        vec![as_total(&game.markets()[&total_market_type(2.5)])],
    );
}

#[test]
fn try_to_add_game_appends_markets_to_existing_groups() {
    let first_game = game_with_markets(
        Platform::Betano,
        vec![
            moneyline_market("betano-moneyline", 2.0, 1.8),
            total_market("betano-total", 2.5, 1.9, 1.9),
        ],
    );
    let second_game = game_with_markets(
        Platform::Betano,
        vec![
            moneyline_market("betclic-moneyline", 2.1, 1.75),
            total_market("betclic-total", 2.5, 2.0, 1.85),
        ],
    );

    let mut cluster = FixtureCluster::new(first_game.clone());

    assert!(cluster.try_to_add_game(second_game.clone()).is_ok());

    assert_eq!(2, cluster.game_count());
    assert_moneyline_group(
        &cluster,
        vec![
            as_moneyline(&first_game.markets()[&MarketType::Moneyline]),
            as_moneyline(&second_game.markets()[&MarketType::Moneyline]),
        ],
    );
    assert_total_group(
        &cluster,
        &total_market_type(2.5),
        vec![
            as_total(&first_game.markets()[&total_market_type(2.5)]),
            as_total(&second_game.markets()[&total_market_type(2.5)]),
        ],
    );
}

#[test]
fn try_to_add_game_creates_a_group_for_new_market_types() {
    let first_game = game_with_markets(
        Platform::Betano,
        vec![moneyline_market("betano-moneyline", 2.0, 1.8)],
    );
    let second_game = game_with_markets(
        Platform::Betano,
        vec![
            moneyline_market("betclic-moneyline", 2.1, 1.75),
            total_market("betclic-total", 2.5, 2.0, 1.85),
        ],
    );

    let mut cluster = FixtureCluster::new(first_game.clone());

    assert!(cluster.try_to_add_game(second_game.clone()).is_ok());

    assert_eq!(2, cluster.game_count());
    assert_moneyline_group(
        &cluster,
        vec![
            as_moneyline(&first_game.markets()[&MarketType::Moneyline]),
            as_moneyline(&second_game.markets()[&MarketType::Moneyline]),
        ],
    );
    assert_total_group(
        &cluster,
        &total_market_type(2.5),
        vec![as_total(&second_game.markets()[&total_market_type(2.5)])],
    );
}

#[test]
fn try_to_add_game_keeps_total_markets_with_different_lines_in_separate_groups() {
    let first_game = game_with_markets(
        Platform::Betano,
        vec![total_market("betano-total", 2.5, 1.9, 1.9)],
    );
    let second_game = game_with_markets(
        Platform::Betano,
        vec![total_market("betclic-total", 3.0, 2.0, 1.85)],
    );

    let mut cluster = FixtureCluster::new(first_game.clone());

    assert!(cluster.try_to_add_game(second_game.clone()).is_ok());

    assert_eq!(2, cluster.game_count());
    assert_eq!(2, cluster.market_type_to_game_ids.len());
    assert_total_group(
        &cluster,
        &total_market_type(2.5),
        vec![as_total(&first_game.markets()[&total_market_type(2.5)])],
    );
    assert_total_group(
        &cluster,
        &total_market_type(3.0),
        vec![as_total(&second_game.markets()[&total_market_type(3.0)])],
    );
}

#[test]
fn arbitrage_opportunites_returns_multiple_arbitrage_types_together() {
    let first_game = game_with_markets(
        Platform::Betano,
        vec![
            moneyline_market("betano-moneyline", 2.2, 1.7),
            total_market("betano-total", 2.5, 2.15, 1.75),
        ],
    );
    let second_game = game_with_markets(
        Platform::Betano,
        vec![
            moneyline_market("betclic-moneyline", 1.8, 2.2),
            total_market("betclic-total", 2.5, 1.8, 2.15),
        ],
    );

    let mut cluster = FixtureCluster::new(first_game);

    assert!(cluster.try_to_add_game(second_game).is_ok());

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

#[test]
fn build_market_group_includes_market_types_added_after_game_is_clustered() {
    let game = game_with_markets(
        Platform::Betano,
        vec![moneyline_market("betano-moneyline", 2.0, 1.8)],
    );
    let game_id = game.id.clone();
    let added_total_market = total_market("betano-total", 2.5, 1.9, 1.9).1;

    let mut cluster = FixtureCluster::new(game.clone());

    assert!(
        cluster
            .try_to_add_game(game_with_markets(
                Platform::Betano,
                vec![moneyline_market("betclic-moneyline", 2.1, 1.75)],
            ))
            .is_ok()
    );

    cluster.update_markets(&game_id, vec![added_total_market.clone()]);

    assert_total_group(
        &cluster,
        &total_market_type(2.5),
        vec![as_total(&added_total_market)],
    );
}

#[test]
fn try_to_add_game_returns_err_and_keeps_cluster_unchanged_for_different_fixture() {
    let first_game = game_with_markets(
        Platform::Betano,
        vec![moneyline_market("betano-moneyline", 2.0, 1.8)],
    );
    let different_game = Game::new(
        "Sporting CP",
        "SC Braga",
        "Portugal",
        "Liga Portugal",
        fixture_date(15, 30),
        Platform::Betano,
        vec![moneyline_market("betclic-moneyline", 2.1, 1.75).1],
    );
    let different_game_id = different_game.id.clone();

    let mut cluster = FixtureCluster::new(first_game.clone());

    let returned_game = cluster.try_to_add_game(different_game).unwrap_err();

    assert_eq!(different_game_id, returned_game.id);
    assert_eq!(1, cluster.game_count());
    assert_moneyline_group(
        &cluster,
        vec![as_moneyline(&first_game.markets()[&MarketType::Moneyline])],
    );
}

#[test]
fn try_to_add_game_does_not_duplicate_existing_game_id() {
    let game = game_with_markets(
        Platform::Betano,
        vec![moneyline_market("betano-moneyline", 2.0, 1.8)],
    );

    let mut cluster = FixtureCluster::new(game.clone());

    assert!(cluster.try_to_add_game(game.clone()).is_ok());

    let game_ids = cluster
        .market_type_to_game_ids
        .get(&MarketType::Moneyline)
        .unwrap();

    assert_eq!(1, cluster.game_count());
    assert_eq!(1, game_ids.len());
    assert!(game_ids.contains(&game.id));
    assert_moneyline_group(
        &cluster,
        vec![as_moneyline(&game.markets()[&MarketType::Moneyline])],
    );
}

#[test]
fn update_markets_ignores_unknown_game_id() {
    let game = game_with_markets(
        Platform::Betano,
        vec![moneyline_market("betano-moneyline", 2.0, 1.8)],
    );
    let added_total_market = total_market("betano-total", 2.5, 1.9, 1.9).1;

    let mut cluster = FixtureCluster::new(game.clone());

    cluster.update_markets("missing-game", vec![added_total_market]);

    assert_eq!(1, cluster.game_count());
    assert!(
        cluster
            .market_type_to_game_ids
            .get(&total_market_type(2.5))
            .is_none()
    );
    assert_moneyline_group(
        &cluster,
        vec![as_moneyline(&game.markets()[&MarketType::Moneyline])],
    );
}

#[test]
fn update_markets_does_not_duplicate_game_ids_for_existing_market_type() {
    let game = game_with_markets(
        Platform::Betano,
        vec![moneyline_market("betano-moneyline", 2.0, 1.8)],
    );
    let game_id = game.id.clone();
    let updated_moneyline = moneyline_market("betano-moneyline-updated", 2.2, 1.7).1;

    let mut cluster = FixtureCluster::new(game);

    cluster.update_markets(&game_id, vec![updated_moneyline.clone()]);

    let game_ids = cluster
        .market_type_to_game_ids
        .get(&MarketType::Moneyline)
        .unwrap();

    assert_eq!(1, game_ids.len());
    assert!(game_ids.contains(&game_id));
    assert_moneyline_group(&cluster, vec![as_moneyline(&updated_moneyline)]);
}

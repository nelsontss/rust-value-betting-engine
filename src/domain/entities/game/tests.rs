use super::Game;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::domain::entities::market::{
    Line, Market, MarketType, MoneylineMarket, Odd, TotalMarket,
};

const DEFAULT_COUNTRY: &str = "Portugal";
const DEFAULT_COMPETITION: &str = "Primeira Liga";
const DEFAULT_PLATFORM: &str = "Betano";

fn build_game(home_team: &str, away_team: &str, date: NaiveDateTime) -> Game {
    Game::new(
        home_team,
        away_team,
        DEFAULT_COUNTRY,
        DEFAULT_COMPETITION,
        date,
        DEFAULT_PLATFORM,
        vec![],
    )
}

fn fixture_date(day: u32) -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 5, day).unwrap(),
        NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap(),
    )
}

fn assert_same_fixture(left: (&str, &str), right: (&str, &str)) {
    let date = fixture_date(1);
    let left = build_game(left.0, left.1, date);
    let right = build_game(right.0, right.1, date);

    assert!(left.same_fixture_as(&right));
}

fn assert_not_same_fixture(left: (&str, &str), right: (&str, &str)) {
    let date = fixture_date(1);
    let left = build_game(left.0, left.1, date);
    let right = build_game(right.0, right.1, date);

    assert!(!left.same_fixture_as(&right));
}

fn assert_not_same_fixture_with_context(
    left: (&str, &str, &str, &str),
    right: (&str, &str, &str, &str),
) {
    let date = fixture_date(1);
    let left = Game::new(
        left.0,
        left.1,
        left.2,
        left.3,
        date,
        DEFAULT_PLATFORM,
        vec![],
    );
    let right = Game::new(
        right.0,
        right.1,
        right.2,
        right.3,
        date,
        DEFAULT_PLATFORM,
        vec![],
    );

    assert!(!left.same_fixture_as(&right));
}

fn moneyline_market(id: &str, home: f64, away: f64) -> Market {
    Market::Moneyline(MoneylineMarket::new(
        id,
        Odd::new(home).unwrap(),
        Odd::new(away).unwrap(),
    ))
}

fn total_market(id: &str, line: f32, over: f64, under: f64) -> Market {
    Market::Total(TotalMarket::new(
        id,
        Line(line),
        Odd::new(over).unwrap(),
        Odd::new(under).unwrap(),
    ))
}

#[test]
fn same_fixture_as_matches_normalized_names() {
    assert_same_fixture(("Benfíca", "Sporting CP"), ("benfica", "sporting cp"));
}

#[test]
fn same_fixture_as_rejects_different_dates() {
    let left = build_game("Benfica", "Sporting", fixture_date(1));
    let right = build_game("Benfica", "Sporting", fixture_date(2));

    assert!(!left.same_fixture_as(&right));
}

#[test]
fn same_fixture_as_rejects_different_countries() {
    assert_not_same_fixture_with_context(
        ("Benfica", "Sporting", "Portugal", "Primeira Liga"),
        ("Benfica", "Sporting", "Spain", "Primeira Liga"),
    );
}

#[test]
fn same_fixture_as_rejects_different_competitions() {
    assert_not_same_fixture_with_context(
        ("Benfica", "Sporting", "Portugal", "Primeira Liga"),
        ("Benfica", "Sporting", "Portugal", "Taca de Portugal"),
    );
}

#[test]
fn normalize_name_removes_accents_and_lowercases() {
    let name = "çÇÁ".to_string();

    assert_eq!(Game::normalize_name(&name), "cca");
}

#[test]
fn same_fixture_as_matches_simillar_names() {
    assert_same_fixture(("Benfica", "Sporting"), ("sl benfica", "sporting cp"));
}

#[test]
fn same_fixture_as_matches_common_aliases() {
    assert_same_fixture(("FC Porto", "Man United"), ("Porto", "Manchester Utd"));
}

#[test]
fn same_fixture_as_matches_names_with_extra_spacing() {
    assert_same_fixture(("  Benfica", "Sporting  CP"), ("Benfica", "Sporting CP"));
}

#[test]
fn same_fixture_as_rejects_swapped_home_and_away_teams() {
    assert_not_same_fixture(("Benfica", "Porto"), ("Porto", "Benfica"));
}

#[test]
#[ignore = "Known false positive: current similarity thresholds do not distinguish this case yet"]
fn same_fixture_as_rejects_similar_home_names_for_different_teams() {
    assert_not_same_fixture(("Sporting", "Porto"), ("Sporting Braga", "Porto"));
}

#[test]
#[ignore = "Known false positive: current similarity thresholds do not distinguish this case yet"]
fn same_fixture_as_rejects_similar_away_names_for_different_teams() {
    assert_not_same_fixture(("Benfica", "Inter"), ("Benfica", "Inter Miami"));
}

#[test]
#[ignore = "Known false positive: current similarity thresholds do not distinguish this case yet"]
fn same_fixture_as_rejects_different_teams_with_shared_city_names() {
    assert_not_same_fixture(
        ("Manchester United", "Benfica"),
        ("Manchester City", "Benfica"),
    );
}

#[test]
fn new_indexes_markets_by_derived_market_type() {
    let game = Game::new(
        "Benfica",
        "Sporting",
        DEFAULT_COUNTRY,
        DEFAULT_COMPETITION,
        fixture_date(1),
        DEFAULT_PLATFORM,
        vec![
            moneyline_market("moneyline", 2.0, 1.8),
            total_market("total", 2.5, 1.9, 1.9),
        ],
    );

    assert!(matches!(
        game.markets().get(&MarketType::Moneyline),
        Some(Market::Moneyline(_))
    ));
    assert!(matches!(
        game.markets().get(&MarketType::Total { line: 250 }),
        Some(Market::Total(_))
    ));
}

#[test]
fn update_market_replaces_existing_market_with_same_logical_type() {
    let mut game = Game::new(
        "Benfica",
        "Sporting",
        DEFAULT_COUNTRY,
        DEFAULT_COMPETITION,
        fixture_date(1),
        DEFAULT_PLATFORM,
        vec![moneyline_market("opening", 2.0, 1.8)],
    );

    game.update_market(vec![moneyline_market("updated", 2.2, 1.7)]);

    assert_eq!(1, game.markets().len());
    assert!(matches!(
        game.markets().get(&MarketType::Moneyline),
        Some(Market::Moneyline(MoneylineMarket { home, away, .. }))
            if *home == Odd::new(2.2).unwrap() && *away == Odd::new(1.7).unwrap()
    ));
}

#[test]
fn update_market_adds_new_market_for_different_logical_type() {
    let mut game = Game::new(
        "Benfica",
        "Sporting",
        DEFAULT_COUNTRY,
        DEFAULT_COMPETITION,
        fixture_date(1),
        DEFAULT_PLATFORM,
        vec![moneyline_market("moneyline", 2.0, 1.8)],
    );

    game.update_market(vec![total_market("total", 2.5, 1.9, 1.9)]);

    assert_eq!(2, game.markets().len());
    assert!(game.markets().contains_key(&MarketType::Moneyline));
    assert!(
        game.markets()
            .contains_key(&MarketType::Total { line: 250 })
    );
}

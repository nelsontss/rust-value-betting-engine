use chrono::{TimeDelta, Utc};
use serde_json::json;

use crate::domain::entities::{Market, Platform};
use crate::infrastructure::parsers::lebull_parser::LeBullParser;

fn today_ms() -> i64 {
    Utc::now().timestamp_millis()
}

fn stake(code: i64, argument: f64, factor: f64) -> serde_json::Value {
    json!({
        "stakeCode": code,
        "stakeArgument": argument,
        "betFactor": factor,
    })
}

fn stakes_1x2(home: f64, draw: f64, away: f64) -> Vec<serde_json::Value> {
    vec![
        stake(1, 0.0, home),
        stake(2, 0.0, draw),
        stake(3, 0.0, away),
    ]
}

fn lebull_event(
    id: i64,
    team_a: &str,
    team_b: &str,
    stake_types: Vec<serde_json::Value>,
    is_live: bool,
) -> serde_json::Value {
    json!({
        "eventId": id,
        "teamA": team_a,
        "teamB": team_b,
        "isLive": is_live,
        "date": format!("/Date({})/", today_ms()),
        "stakeTypes": stake_types,
    })
}

fn league(country: &str, league_name: &str, games: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
        "countryName": country,
        "leagueName": league_name,
        "games": games,
    })
}

fn count_by_type(markets: &[Market]) -> (usize, usize, usize, usize) {
    let mut match_result = 0;
    let mut moneyline = 0;
    let mut total = 0;
    let mut asian = 0;
    for m in markets {
        match m {
            Market::MatchResult(_) => match_result += 1,
            Market::Moneyline(_) => moneyline += 1,
            Market::Total(_) => total += 1,
            Market::AsianHandicap(_) => asian += 1,
            _ => {}
        }
    }
    (match_result, moneyline, total, asian)
}

#[test]
fn parse_data_non_array_returns_empty_vec() {
    let result = LeBullParser::parse_data(json!({"key": "value"}));
    assert!(result.is_empty());
}

#[test]
fn parse_data_empty_array_returns_empty_vec() {
    let result = LeBullParser::parse_data(json!([]));
    assert!(result.is_empty());
}

#[test]
fn parse_data_league_without_games_skips() {
    let data = json!([{"countryName": "Portugal", "leagueName": "Liga"}]);
    let result = LeBullParser::parse_data(data);
    assert!(result.is_empty());
}

#[test]
fn parse_data_parses_1x2_market() {
    let events = vec![lebull_event(
        100,
        "FC Porto",
        "SL Benfica",
        vec![json!({
            "stakeTypeId": 1,
            "stakes": stakes_1x2(2.0, 3.2, 4.0),
        })],
        false,
    )];
    let data = json!([league("Portugal", "Liga Portugal", events)]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].home_team(), "FC Porto");
    assert_eq!(games[0].away_team(), "SL Benfica");
    assert_eq!(games[0].platform(), Platform::LeBull);
    let (mr, ml, t, ah) = count_by_type(&games[0].markets().values().cloned().collect::<Vec<_>>());
    assert_eq!(mr, 1);
    assert_eq!(ml, 0);
    assert_eq!(t, 0);
    assert_eq!(ah, 0);
}

#[test]
fn parse_data_parses_asian_handicap_market() {
    let events = vec![lebull_event(
        101,
        "Sporting",
        "Braga",
        vec![json!({
            "stakeTypeId": 2,
            "stakes": vec![
                stake(1, -0.5, 2.0),
                stake(2, 0.5, 1.8),
            ],
        })],
        false,
    )];
    let data = json!([league("Portugal", "Liga Portugal", events)]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
    let (mr, _ml, _t, ah) =
        count_by_type(&games[0].markets().values().cloned().collect::<Vec<_>>());
    assert_eq!(ah, 1);
    assert_eq!(mr, 0);
}

#[test]
fn parse_data_parses_total_market() {
    let events = vec![lebull_event(
        102,
        "Benfica",
        "Porto",
        vec![json!({
            "stakeTypeId": 3,
            "stakes": vec![
                stake(1, 2.5, 1.9),
                stake(2, 2.5, 1.9),
            ],
        })],
        false,
    )];
    let data = json!([league("Portugal", "Liga Portugal", events)]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
    let (_mr, _ml, t, _ah) =
        count_by_type(&games[0].markets().values().cloned().collect::<Vec<_>>());
    assert_eq!(t, 1);
}

#[test]
fn parse_data_parses_2way_market_type_26() {
    let events = vec![lebull_event(
        103,
        "Arsenal",
        "Chelsea",
        vec![json!({
            "stakeTypeId": 26,
            "stakes": vec![
                stake(1, 0.0, 1.8),
                stake(2, 0.0, 2.1),
            ],
        })],
        false,
    )];
    let data = json!([league("England", "Premier League", events)]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
    let (_mr, ml, _t, _ah) =
        count_by_type(&games[0].markets().values().cloned().collect::<Vec<_>>());
    assert_eq!(ml, 1);
}

#[test]
fn parse_data_parses_2way_market_type_274556() {
    let events = vec![lebull_event(
        104,
        "Milan",
        "Inter",
        vec![json!({
            "stakeTypeId": 274556,
            "stakes": vec![
                stake(1, 0.0, 2.2),
                stake(2, 0.0, 1.7),
            ],
        })],
        false,
    )];
    let data = json!([league("Italy", "Serie A", events)]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
    let (_mr, ml, _t, _ah) =
        count_by_type(&games[0].markets().values().cloned().collect::<Vec<_>>());
    assert_eq!(ml, 1);
}

#[test]
fn parse_data_skips_live_events() {
    let live_events = vec![lebull_event(
        200,
        "Team A",
        "Team B",
        vec![json!({
            "stakeTypeId": 1,
            "stakes": stakes_1x2(2.0, 3.2, 4.0),
        })],
        true,
    )];
    let data = json!([league("Portugal", "Liga", live_events)]);

    let games = LeBullParser::parse_data(data);
    assert!(games.is_empty());
}

#[test]
fn parse_data_skips_unknown_stake_type() {
    let events = vec![lebull_event(
        300,
        "Team A",
        "Team B",
        vec![json!({
            "stakeTypeId": 999,
            "stakes": stakes_1x2(2.0, 3.2, 4.0),
        })],
        false,
    )];
    let data = json!([league("Portugal", "Liga", events)]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert!(games[0].markets().is_empty());
}

#[test]
fn parse_data_handles_event_without_stake_types() {
    let event = json!({
        "eventId": 400,
        "teamA": "Team A",
        "teamB": "Team B",
        "isLive": false,
        "date": format!("/Date({})/", today_ms()),
    });
    let data = json!([league("Portugal", "Liga", vec![event])]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert!(games[0].markets().is_empty());
}

#[test]
fn parse_data_multiple_leagues() {
    let events_1 = vec![lebull_event(
        500,
        "Porto",
        "Benfica",
        vec![json!({
            "stakeTypeId": 1,
            "stakes": stakes_1x2(2.0, 3.2, 4.0),
        })],
        false,
    )];
    let events_2 = vec![lebull_event(
        501,
        "Sporting",
        "Braga",
        vec![json!({
            "stakeTypeId": 1,
            "stakes": stakes_1x2(1.8, 3.4, 4.5),
        })],
        false,
    )];
    let data = json!([
        league("Portugal", "Liga Portugal", events_1),
        league("Portugal", "Liga Portugal", events_2),
    ]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 2);
}

#[test]
fn parse_data_parses_date_from_timestamp_when_date_missing() {
    let event = json!({
        "eventId": 600,
        "teamA": "Team A",
        "teamB": "Team B",
        "isLive": false,
        "timestamp": today_ms(),
        "stakeTypes": [],
    });
    let data = json!([league("Portugal", "Liga", vec![event])]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
}

#[test]
fn parse_data_with_total_market_multiple_lines() {
    let events = vec![lebull_event(
        700,
        "Team X",
        "Team Y",
        vec![json!({
            "stakeTypeId": 3,
            "stakes": vec![
                stake(1, 2.5, 1.9),
                stake(2, 2.5, 1.9),
                stake(1, 3.0, 2.0),
                stake(2, 3.0, 1.8),
            ],
        })],
        false,
    )];
    let data = json!([league("Portugal", "Liga", events)]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
    let (_, _, t, _) = count_by_type(&games[0].markets().values().cloned().collect::<Vec<_>>());
    assert_eq!(t, 2);
}

#[test]
fn parse_data_with_asian_handicap_multiple_lines() {
    let events = vec![lebull_event(
        701,
        "Team M",
        "Team N",
        vec![json!({
            "stakeTypeId": 2,
            "stakes": vec![
                stake(1, 0.0, 1.9),
                stake(2, 0.0, 1.9),
                stake(1, -0.5, 2.0),
                stake(2, 0.5, 1.8),
            ],
        })],
        false,
    )];
    let data = json!([league("Portugal", "Liga", events)]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
    let (_, _, _, ah) = count_by_type(&games[0].markets().values().cloned().collect::<Vec<_>>());
    assert_eq!(ah, 2);
}

#[test]
fn parse_event_with_tomorrow_date_is_included() {
    let tomorrow = Utc::now() + TimeDelta::days(1);
    let ts = tomorrow.timestamp_millis();
    let event = json!({
        "eventId": 800,
        "teamA": "Team A",
        "teamB": "Team B",
        "isLive": false,
        "date": format!("/Date({})/", ts),
        "stakeTypes": [{"stakeTypeId": 1, "stakes": stakes_1x2(2.0, 3.2, 4.0)}],
    });
    let data = json!([league("Portugal", "Liga", vec![event])]);

    let games = LeBullParser::parse_data(data);
    assert_eq!(games.len(), 1);
}

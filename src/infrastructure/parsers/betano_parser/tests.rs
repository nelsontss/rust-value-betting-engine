use serde_json::json;

use crate::domain::entities::{Market, Platform};
use crate::infrastructure::parsers::betano_parser::BetanoParser;

fn market_type_ids(markets: &[Market]) -> Vec<i64> {
    markets
        .iter()
        .map(|m| match m {
            Market::MatchResult(_) => 1,
            Market::Moneyline(_) => 10,
            Market::Total(_) => 13,
            _ => panic!("unexpected market type"),
        })
        .collect()
}

fn moneyline_count(markets: &[Market]) -> usize {
    markets
        .iter()
        .filter(|m| matches!(m, Market::Moneyline(_)))
        .count()
}

fn match_result_count(markets: &[Market]) -> usize {
    markets
        .iter()
        .filter(|m| matches!(m, Market::MatchResult(_)))
        .count()
}

fn total_count(markets: &[Market]) -> usize {
    markets
        .iter()
        .filter(|m| matches!(m, Market::Total(_)))
        .count()
}

fn selection(price: f64) -> serde_json::Value {
    json!({"price": price})
}

fn market_with_selections(type_id: i64, selections: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
        "typeId": type_id,
        "id": format!("market-{}", type_id),
        "handicap": 0.0,
        "selections": selections,
    })
}

fn betano_event(
    id: &str,
    home: &str,
    away: &str,
    markets: Vec<serde_json::Value>,
) -> serde_json::Value {
    json!({
        "id": id,
        "name": format!("{} - {}", home, away),
        "leagueName": "Liga Portugal",
        "regionName": "Portugal",
        "startTime": 1_777_000_000_000i64,
        "markets": markets,
    })
}

fn block_with_events(events: Vec<serde_json::Value>) -> serde_json::Value {
    json!({"events": events})
}

fn assert_game_counts(
    games: &[crate::domain::Game],
    event_ids: &[&str],
) {
    assert_eq!(games.len(), event_ids.len());
    for (game, expected_id) in games.iter().zip(event_ids) {
        assert_eq!(game.id, *expected_id);
    }
}

#[test]
fn parse_data_empty_json_returns_empty_vec() {
    let result = BetanoParser::parse_data(json!({}));
    assert!(result.is_empty());
}

#[test]
fn parse_data_no_blocks_returns_empty_vec() {
    let result = BetanoParser::parse_data(json!({"blocks": []}));
    assert!(result.is_empty());
}

#[test]
fn parse_data_block_with_no_events_returns_empty_vec() {
    let data = json!({"blocks": [{"other": "data"}]});
    let result = BetanoParser::parse_data(data);
    assert!(result.is_empty());
}

#[test]
fn parse_data_block_with_empty_events_returns_empty_vec() {
    let data = json!({"blocks": [{"events": []}]});
    let result = BetanoParser::parse_data(data);
    assert!(result.is_empty());
}

#[test]
fn parse_data_skips_event_without_dash_separator() {
    let data = json!({
        "blocks": [{
            "events": [{
                "id": "1",
                "name": "NoDashSeparator",
                "leagueName": "Liga",
                "regionName": "Portugal",
                "startTime": 1_777_000_000_000i64,
                "markets": []
            }]
        }]
    });
    let result = BetanoParser::parse_data(data);
    assert!(result.is_empty());
}

#[test]
fn parse_data_match_result_market_type_1() {
    let markets = vec![market_with_selections(1, vec![
        selection(2.0),
        selection(3.2),
        selection(4.0),
    ])];
    let events = vec![betano_event("evt-1", "FC Porto", "SL Benfica", markets)];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].home_team(), "FC Porto");
    assert_eq!(games[0].away_team(), "SL Benfica");
    assert_eq!(games[0].platform(), Platform::Betano);
    assert_eq!(match_result_count(&games[0].markets().values().cloned().collect::<Vec<_>>()), 1);
}

#[test]
fn parse_data_type_10_is_moneyline() {
    let markets = vec![market_with_selections(10, vec![
        selection(1.8),
        selection(2.1),
    ])];
    let events = vec![betano_event("evt-2", "Benfica", "Porto", markets)];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(moneyline_count(&games[0].markets().values().cloned().collect::<Vec<_>>()), 1);
}

#[test]
fn parse_data_type_13_is_total() {
    let markets = vec![market_with_selections(13, vec![
        selection(1.9),
        selection(1.9),
    ])];
    let events = vec![betano_event("evt-3", "Sporting", "Braga", markets)];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(total_count(&games[0].markets().values().cloned().collect::<Vec<_>>()), 1);
}

#[test]
fn parse_data_type_14_is_first_half_total() {
    let markets = vec![market_with_selections(14, vec![
        selection(2.0),
        selection(1.8),
    ])];
    let events = vec![betano_event("evt-4", "Arsenal", "Chelsea", markets)];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(total_count(&games[0].markets().values().cloned().collect::<Vec<_>>()), 1);
}

#[test]
fn parse_data_type_15_is_skipped() {
    let markets = vec![
        market_with_selections(1, vec![selection(2.0), selection(3.2), selection(4.0)]),
        market_with_selections(15, vec![selection(1.5), selection(2.5)]),
    ];
    let events = vec![betano_event("evt-5", "Liverpool", "Everton", markets)];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert_eq!(games.len(), 1);
    let types = market_type_ids(&games[0].markets().values().cloned().collect::<Vec<_>>());
    assert_eq!(types, vec![1]);
}

#[test]
fn parse_data_multiple_markets_in_one_event() {
    let markets = vec![
        market_with_selections(1, vec![selection(2.0), selection(3.2), selection(4.0)]),
        market_with_selections(13, vec![selection(1.9), selection(1.9)]),
        market_with_selections(10, vec![selection(1.8), selection(2.1)]),
    ];
    let events = vec![betano_event("evt-6", "Milan", "Inter", markets)];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert_eq!(games.len(), 1);
    let all_markets = games[0].markets().values().cloned().collect::<Vec<_>>();
    assert_eq!(match_result_count(&all_markets), 1);
    assert_eq!(total_count(&all_markets), 1);
    assert_eq!(moneyline_count(&all_markets), 1);
}

#[test]
fn parse_data_unknown_type_id_is_skipped() {
    let markets = vec![
        market_with_selections(999, vec![selection(2.0), selection(3.0)]),
        market_with_selections(1, vec![selection(2.0), selection(3.2), selection(4.0)]),
    ];
    let events = vec![betano_event("evt-7", "Barcelona", "Madrid", markets)];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert_eq!(games.len(), 1);
    let types = market_type_ids(&games[0].markets().values().cloned().collect::<Vec<_>>());
    assert_eq!(types, vec![1]);
}

#[test]
fn parse_data_event_without_markets_still_creates_game() {
    let events = vec![betano_event("evt-8", "Juventus", "Napoli", vec![])];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert!(games[0].markets().is_empty());
}

#[test]
fn parse_data_multiple_events() {
    let markets = vec![market_with_selections(1, vec![
        selection(2.0),
        selection(3.2),
        selection(4.0),
    ])];
    let events = vec![
        betano_event("evt-a", "Team A", "Team B", markets.clone()),
        betano_event("evt-b", "Team C", "Team D", markets.clone()),
        betano_event("evt-c", "Team E", "Team F", markets),
    ];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert_game_counts(&games, &["evt-a", "evt-b", "evt-c"]);
}

#[test]
fn parse_data_type_10_requires_at_least_two_selections() {
    let markets = vec![market_with_selections(10, vec![selection(1.5)])];
    let events = vec![betano_event("evt-10", "Team A", "Team B", markets)];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert!(games[0].markets().is_empty());
}

#[test]
fn parse_data_handles_handicap_value_correctly_for_totals() {
    let markets = vec![
        market_with_selections(13, vec![selection(1.9), selection(1.9)]),
    ];
    let mut market_with_line = markets[0].clone();
    if let Some(obj) = market_with_line.as_object() {
        let mut obj = obj.clone();
        obj.insert("handicap".to_string(), json!(2.5));
        market_with_line = serde_json::Value::Object(obj);
    }
    let events = vec![betano_event("evt-h", "Team G", "Team H", vec![market_with_line])];
    let data = json!({"blocks": [block_with_events(events)]});

    let result = BetanoParser::parse_data(data);
    assert_eq!(result.len(), 1);
}

#[test]
fn parse_data_type_1_requires_at_least_three_selections() {
    let markets = vec![market_with_selections(1, vec![selection(2.0), selection(3.0)])];
    let events = vec![betano_event("evt-11", "Team A", "Team B", markets)];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert!(games[0].markets().is_empty());
}

#[test]
fn parse_data_type_13_requires_at_least_two_selections() {
    let markets = vec![market_with_selections(13, vec![selection(1.5)])];
    let events = vec![betano_event("evt-12", "Team A", "Team B", markets)];
    let data = json!({"blocks": [block_with_events(events)]});

    let games = BetanoParser::parse_data(data);
    assert!(games[0].markets().is_empty());
}

#[test]
fn parse_data_missing_league_name_uses_empty_string() {
    let markets = vec![market_with_selections(1, vec![
        selection(2.0),
        selection(3.2),
        selection(4.0),
    ])];
    let event = json!({
        "id": "evt-no-league",
        "name": "Team A - Team B",
        "regionName": "Portugal",
        "startTime": 1_777_000_000_000i64,
        "markets": markets,
    });
    let data = json!({"blocks": [{"events": [event]}]});

    let games = BetanoParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].competition(), "");
}

#[test]
fn parse_data_multiple_blocks() {
    let markets = vec![market_with_selections(1, vec![
        selection(2.0),
        selection(3.2),
        selection(4.0),
    ])];
    let events_a = vec![betano_event("evt-b1", "Team A", "Team B", markets.clone())];
    let events_b = vec![betano_event("evt-b2", "Team C", "Team D", markets)];
    let data = json!({"blocks": [block_with_events(events_a), block_with_events(events_b)]});

    let games = BetanoParser::parse_data(data);
    assert_game_counts(&games, &["evt-b1", "evt-b2"]);
}

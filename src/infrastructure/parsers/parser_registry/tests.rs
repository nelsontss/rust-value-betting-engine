use chrono::Utc;
use serde_json::json;

use crate::domain::entities::Platform;
use crate::infrastructure::parsers::parser_registry::ParserRegistry;

fn today_ms() -> i64 {
    Utc::now().timestamp_millis()
}

#[test]
fn new_registers_betano_and_lebull_parsers() {
    let registry = ParserRegistry::new();
    let betano_result = registry.parse(
        &Platform::Betano,
        json!({"blocks": [{"events": []}]}),
    );
    assert!(betano_result.is_some());
    assert_eq!(betano_result.unwrap().len(), 0);

    let lebull_result = registry.parse(&Platform::LeBull, json!([]));
    assert!(lebull_result.is_some());
    assert_eq!(lebull_result.unwrap().len(), 0);
}

#[test]
fn parse_unknown_platform_returns_none() {
    let registry = ParserRegistry::new();
    let result = registry.parse(
        &Platform::Betano,
        json!({"blocks": [{"events": [{"id": "1", "name": "A - B", "leagueName": "L", "regionName": "R", "startTime": 1_777_000_000_000i64, "markets": []}]}]}),
    );
    assert!(result.is_some());
}

#[test]
fn parse_betano_data_returns_games() {
    let registry = ParserRegistry::new();
    let data = json!({
        "blocks": [{
            "events": [{
                "id": "evt-1",
                "name": "FC Porto - SL Benfica",
                "leagueName": "Liga Portugal",
                "regionName": "Portugal",
                "startTime": 1_777_000_000_000i64,
                "markets": [{
                    "typeId": 1,
                    "id": "m1",
                    "handicap": 0.0,
                    "selections": [
                        {"price": 2.0},
                        {"price": 3.2},
                        {"price": 4.0},
                    ],
                }],
            }]
        }]
    });

    let result = registry.parse(&Platform::Betano, data);
    assert!(result.is_some());
    let games = result.unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].home_team(), "FC Porto");
    assert_eq!(games[0].platform(), Platform::Betano);
}

#[test]
fn parse_lebull_data_returns_games() {
    let registry = ParserRegistry::new();
    let data = json!([{
        "countryName": "Portugal",
        "leagueName": "Liga Portugal",
        "games": [{
            "eventId": 100,
            "teamA": "FC Porto",
            "teamB": "SL Benfica",
            "isLive": false,
            "date": format!("/Date({})/", today_ms()),
            "stakeTypes": [{
                "stakeTypeId": 1,
                "stakes": [
                    {"stakeCode": 1, "stakeArgument": 0.0, "betFactor": 2.0},
                    {"stakeCode": 2, "stakeArgument": 0.0, "betFactor": 3.2},
                    {"stakeCode": 3, "stakeArgument": 0.0, "betFactor": 4.0},
                ],
            }],
        }]
    }]);

    let result = registry.parse(&Platform::LeBull, data);
    assert!(result.is_some());
    let games = result.unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].home_team(), "FC Porto");
    assert_eq!(games[0].platform(), Platform::LeBull);
}

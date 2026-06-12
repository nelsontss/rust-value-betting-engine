use serde_json::{json, Value};

use crate::domain::entities::{Market, Platform};
use crate::infrastructure::parsers::bwin_parser::{BwinParser, BwinWSEvent, SignalRFrame};

fn make_option(params: Vec<(&str, &str)>, options: Vec<Value>) -> Value {
    let mut om = json!({
        "id": 12345,
        "status": "Visible",
        "parameters": params.iter().map(|(k, v)| json!({"key": k, "value": v})).collect::<Vec<_>>(),
    });
    if !options.is_empty() {
        om["options"] = json!(options);
    }
    om
}

fn regular_3way(home: f64, draw: f64, away: f64) -> Value {
    make_option(
        vec![("Period", "RegularTime"), ("MarketType", "3way")],
        vec![
            json!({"price": {"odds": home}}),
            json!({"price": {"odds": draw}}),
            json!({"price": {"odds": away}}),
        ],
    )
}

fn regular_double_chance(hd: f64, da: f64, ha: f64) -> Value {
    make_option(
        vec![("Period", "RegularTime"), ("MarketType", "DoubleChance")],
        vec![
            json!({"price": {"odds": hd}}),
            json!({"price": {"odds": da}}),
            json!({"price": {"odds": ha}}),
        ],
    )
}

fn regular_over_under(line: f32, over: f64, under: f64) -> Value {
    make_option(
        vec![
            ("Period", "RegularTime"),
            ("MarketType", "Over/Under"),
            ("DecimalValue", &line.to_string()),
        ],
        vec![
            json!({"price": {"odds": over}}),
            json!({"price": {"odds": under}}),
        ],
    )
}

fn regular_handicap(line: f32, home: f64, draw: f64, away: f64) -> Value {
    make_option(
        vec![
            ("Period", "RegularTime"),
            ("MarketType", "Handicap"),
            ("Handicap", &line.to_string()),
        ],
        vec![
            json!({"price": {"odds": home}}),
            json!({"price": {"odds": draw}}),
            json!({"price": {"odds": away}}),
        ],
    )
}

fn regular_draw_no_bet(home: f64, away: f64) -> Value {
    make_option(
        vec![("Period", "RegularTime"), ("MarketType", "DrawNoBet")],
        vec![
            json!({"price": {"odds": home}}),
            json!({"price": {"odds": away}}),
        ],
    )
}

fn fixture(id: &str, option_markets: Vec<Value>) -> Value {
    json!({
        "id": id,
        "participants": [
            {"properties": {"type": "HomeTeam"}, "name": {"value": "FC Porto"}},
            {"properties": {"type": "AwayTeam"}, "name": {"value": "SL Benfica"}},
        ],
        "competition": {"name": {"value": "Liga Portugal"}},
        "region": {"name": {"value": "Portugal"}},
        "startDate": "2026-06-12T20:00:00Z",
        "optionMarkets": option_markets,
    })
}

// --- parse_data tests ---

#[test]
fn parse_data_parses_3way_as_match_result() {
    let data = json!({"fixtures": [fixture("f1", vec![regular_3way(2.0, 3.2, 4.0)])]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].home_team(), "FC Porto");
    assert_eq!(games[0].away_team(), "SL Benfica");
    assert_eq!(games[0].platform(), Platform::Bwin);
    assert_eq!(games[0].markets().len(), 1);
    assert!(matches!(
        games[0].markets().values().next().unwrap(),
        Market::MatchResult(_)
    ));
}

#[test]
fn parse_data_parses_double_chance() {
    let data = json!({"fixtures": [fixture("f1", vec![regular_double_chance(1.5, 2.0, 3.0)])]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].markets().len(), 1);
    assert!(matches!(
        games[0].markets().values().next().unwrap(),
        Market::DoubleChance(_)
    ));
}

#[test]
fn parse_data_parses_over_under_as_total() {
    let data = json!({"fixtures": [fixture("f1", vec![regular_over_under(2.5, 1.9, 1.9)])]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].markets().len(), 1);
    assert!(matches!(
        games[0].markets().values().next().unwrap(),
        Market::Total(_)
    ));
}

#[test]
fn parse_data_parses_handicap() {
    let data = json!({"fixtures": [fixture("f1", vec![regular_handicap(-1.0, 2.5, 3.3, 2.8)])]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].markets().len(), 1);
    assert!(matches!(
        games[0].markets().values().next().unwrap(),
        Market::Handicap(_)
    ));
}

#[test]
fn parse_data_parses_draw_no_bet_as_moneyline() {
    let data = json!({"fixtures": [fixture("f1", vec![regular_draw_no_bet(1.8, 2.0)])]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].markets().len(), 1);
    assert!(matches!(
        games[0].markets().values().next().unwrap(),
        Market::Moneyline(_)
    ));
}

#[test]
fn parse_data_mixed_market_types() {
    let data = json!({"fixtures": [fixture("f1", vec![
        regular_3way(2.0, 3.2, 4.0),
        regular_over_under(2.5, 1.9, 1.9),
        regular_double_chance(1.5, 2.0, 3.0),
    ])]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].markets().len(), 3);
}

#[test]
fn parse_data_empty_fixtures_returns_empty() {
    let data = json!({"fixtures": []});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 0);
}

#[test]
fn parse_data_missing_fixtures_field_returns_empty() {
    let data = json!({});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 0);
}

#[test]
fn parse_data_missing_participants_skips_fixture() {
    let data = json!({"fixtures": [{"id": "f1", "startDate": "2026-06-12T20:00:00Z"}]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 0);
}

#[test]
fn parse_data_missing_start_date_skips_fixture() {
    let data = json!({"fixtures": [{"id": "f1", "participants": [
        {"properties": {"type": "HomeTeam"}, "name": {"value": "A"}},
        {"properties": {"type": "AwayTeam"}, "name": {"value": "B"}},
    ]}]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 0);
}

#[test]
fn parse_data_empty_option_markets_creates_game_with_no_markets() {
    let data = json!({"fixtures": [fixture("f1", vec![])]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].markets().len(), 0);
}

#[test]
fn parse_data_filters_non_visible_option_markets() {
    let mut om = regular_3way(2.0, 3.2, 4.0);
    om["status"] = json!("Suspended");
    let data = json!({"fixtures": [fixture("f1", vec![om])]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].markets().len(), 0);
}

#[test]
fn parse_data_filters_non_regular_time_period() {
    let mut om = regular_3way(2.0, 3.2, 4.0);
    om["parameters"][0]["value"] = json!("FirstHalf");
    let data = json!({"fixtures": [fixture("f1", vec![om])]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].markets().len(), 0);
}

#[test]
fn parse_data_filters_unknown_market_type() {
    let om = make_option(
        vec![("Period", "RegularTime"), ("MarketType", "UnknownType")],
        vec![json!({"price": {"odds": 2.0}})],
    );
    let data = json!({"fixtures": [fixture("f1", vec![om])]});
    let games = BwinParser::parse_data(data);
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].markets().len(), 0);
}

// --- BwinWSEvent tests ---

#[test]
fn parse_ws_event_ping() {
    let event = BwinParser::parse_ws_event(r#"{"type":6}"#).unwrap();
    assert!(matches!(event, BwinWSEvent::Ping));
}

#[test]
fn parse_ws_event_connection_ack() {
    let data = r#"{"type":1,"target":"Receive","invocationId":"0","arguments":[{"messageType":"ConnectionAck","payload":{"connectionId":"conn-123"}}]}"#;
    let event = BwinParser::parse_ws_event(data).unwrap();
    match event {
        BwinWSEvent::ConnectionAck { connection_id } => {
            assert_eq!(connection_id, "conn-123");
        }
        _ => panic!("expected ConnectionAck"),
    }
}

#[test]
fn parse_ws_event_option_market_update() {
    let data = r#"{"type":1,"target":"Receive","invocationId":"0","arguments":[{"messageType":"OptionMarketUpdate","fixtureId":"f1","payload":{"optionMarket":{"id":1,"status":"Visible","parameters":[{"key":"Period","value":"RegularTime"},{"key":"MarketType","value":"3way"}],"options":[{"price":{"odds":2.0}},{"price":{"odds":3.2}},{"price":{"odds":4.0}}]}}}]}"#;
    let event = BwinParser::parse_ws_event(data).unwrap();
    match event {
        BwinWSEvent::OptionMarketUpdate { fixture_id, .. } => {
            assert_eq!(fixture_id, "f1");
        }
        _ => panic!("expected OptionMarketUpdate"),
    }
}

#[test]
fn parse_ws_event_option_market_delete() {
    let data = r#"{"type":1,"target":"Receive","invocationId":"0","arguments":[{"messageType":"OptionMarketDelete","payload":{"marketId":42,"fixtureId":"f1"}}]}"#;
    let event = BwinParser::parse_ws_event(data).unwrap();
    match event {
        BwinWSEvent::OptionMarketDelete { market_id, fixture_id } => {
            assert_eq!(market_id, 42);
            assert_eq!(fixture_id, "f1");
        }
        _ => panic!("expected OptionMarketDelete"),
    }
}

#[test]
fn parse_ws_event_fixture_update() {
    let data = r#"{"type":1,"target":"Receive","invocationId":"0","arguments":[{"messageType":"FixtureUpdate","fixtureId":"f1","payload":{"stage":"InPlay"}}]}"#;
    let event = BwinParser::parse_ws_event(data).unwrap();
    match event {
        BwinWSEvent::FixtureUpdate { fixture_id, stage } => {
            assert_eq!(fixture_id, "f1");
            assert_eq!(stage, "InPlay");
        }
        _ => panic!("expected FixtureUpdate"),
    }
}

#[test]
fn parse_ws_event_scoreboard_slim() {
    let data = r#"{"type":1,"target":"Receive","invocationId":"0","arguments":[{"messageType":"ScoreboardSlim","fixtureId":"f1","payload":{"homeScore":2,"awayScore":1}}]}"#;
    let event = BwinParser::parse_ws_event(data).unwrap();
    match event {
        BwinWSEvent::ScoreboardSlim { fixture_id, .. } => {
            assert_eq!(fixture_id, "f1");
        }
        _ => panic!("expected ScoreboardSlim"),
    }
}

#[test]
fn parse_ws_event_main_to_live_update() {
    let data = r#"{"type":1,"target":"Receive","invocationId":"0","arguments":[{"messageType":"MainToLiveUpdate","payload":[{"preMatchId":"pm1","inPlayId":"ip1"}]}]}"#;
    let event = BwinParser::parse_ws_event(data).unwrap();
    match event {
        BwinWSEvent::MainToLiveUpdate { switched_fixtures } => {
            assert_eq!(switched_fixtures.len(), 1);
            assert_eq!(switched_fixtures[0].pre_match_id, "pm1");
            assert_eq!(switched_fixtures[0].in_play_id, "ip1");
        }
        _ => panic!("expected MainToLiveUpdate"),
    }
}

#[test]
fn parse_ws_event_unknown_message_type_returns_none() {
    let data = r#"{"type":1,"target":"Receive","invocationId":"0","arguments":[{"messageType":"UnknownType","payload":{}}]}"#;
    let result = BwinParser::parse_ws_event(data);
    assert!(result.is_none());
}

#[test]
fn parse_ws_event_no_arguments_returns_none() {
    let data = r#"{"type":1,"target":"Receive","invocationId":"0"}"#;
    let result = BwinParser::parse_ws_event(data);
    assert!(result.is_none());
}

#[test]
fn parse_ws_event_invalid_json_returns_none() {
    let result = BwinParser::parse_ws_event("not json");
    assert!(result.is_none());
}

#[test]
fn subscribe_frame_generates_correct_signalr_frame() {
    let topics = vec!["v2|pt|4_12345_67_any|grd".into()];
    let frame = BwinWSEvent::subscribe(topics);
    assert_eq!(frame.msg_type, 1);
    assert_eq!(frame.target, Some("Subscribe".into()));
    assert_eq!(frame.invocation_id, Some("0".into()));
    let args = frame.arguments.unwrap();
    let topics_arg = args[0]["topics"].as_array().unwrap();
    assert_eq!(topics_arg[0], "v2|pt|4_12345_67_any|grd");
}

// --- parse_option_market_update tests ---

#[test]
fn parse_option_market_update_returns_market() {
    let data = json!({"optionMarket": regular_3way(2.0, 3.2, 4.0)});
    let markets = BwinParser::parse_option_market_update(data);
    assert_eq!(markets.len(), 1);
    assert!(matches!(markets[0], Market::MatchResult(_)));
}

#[test]
fn parse_option_market_update_missing_option_market_returns_empty() {
    let data = json!({});
    let markets = BwinParser::parse_option_market_update(data);
    assert_eq!(markets.len(), 0);
}

#[test]
fn parse_option_market_update_non_visible_returns_empty() {
    let mut om = regular_3way(2.0, 3.2, 4.0);
    om["status"] = json!("Suspended");
    let data = json!({"optionMarket": om});
    let markets = BwinParser::parse_option_market_update(data);
    assert_eq!(markets.len(), 0);
}

// --- from_frame tests ---

#[test]
fn from_frame_type_6_is_ping() {
    let frame = SignalRFrame {
        msg_type: 6,
        target: None,
        invocation_id: None,
        arguments: None,
    };
    let event = BwinWSEvent::from_frame(frame);
    assert!(matches!(event, Some(BwinWSEvent::Ping)));
}

#[test]
fn from_frame_type_0_returns_none() {
    let frame = SignalRFrame {
        msg_type: 0,
        target: None,
        invocation_id: None,
        arguments: None,
    };
    assert!(BwinWSEvent::from_frame(frame).is_none());
}

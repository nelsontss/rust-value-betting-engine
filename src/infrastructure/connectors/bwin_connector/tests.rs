use crate::domain::entities::{Market, Odd, markets::moneyline::MoneylineMarket};

use super::*;

fn moneyline() -> Market {
    Market::Moneyline(MoneylineMarket::new(
        "ml-1".to_string(),
        Odd::new(2.0).unwrap(),
        Odd::new(1.8).unwrap(),
    ))
}

#[test]
fn get_subscription_topics_builds_fixture_sbs_and_market_topics() {
    let mut game = Game::new(
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            chrono::NaiveTime::from_hms_milli_opt(18, 0, 0, 0).unwrap(),
        ),
        Platform::Bwin,
        vec![moneyline()],
        None,
    );
    game.id = "fixture-42".to_string();

    let topics = BwinConnector::get_subscription_topics(&[game]);

    assert_eq!(
        vec![
            "v2|pt|fixture-42_67_any|fxt".to_string(),
            "v2|pt|fixture-42_67_any|sbs".to_string(),
            "v2|pt|fixture-42_67_any|fxm-ml-1".to_string(),
        ],
        topics
    );
}

#[test]
fn market_bwin_id_reads_the_underlying_market_id() {
    assert_eq!("ml-1", BwinConnector::market_bwin_id(&moneyline()));
}

#[test]
fn headers_contain_browser_like_fingerprint() {
    let headers = BwinConnector::headers();

    assert_eq!(
        "https://www.bwin.pt",
        headers.get("Origin").unwrap().to_str().unwrap()
    );
    assert!(headers.get("User-Agent").is_none());
    assert_eq!(
        "host-app",
        headers.get("X-From-Product").unwrap().to_str().unwrap()
    );
}

#[test]
fn client_builds_with_browser_user_agent() {
    let client = BwinConnector::client();
    let _ = client; // building without panicking is the contract
}

type BlockingWs = tokio_tungstenite::tungstenite::WebSocket<
    tokio_tungstenite::tungstenite::stream::MaybeTlsStream<std::net::TcpStream>,
>;

fn ws_pair() -> (BlockingWs, BlockingWs) {
    use tokio_tungstenite::tungstenite::protocol::Role;
    use tokio_tungstenite::tungstenite::stream::MaybeTlsStream;
    use tokio_tungstenite::tungstenite::WebSocket;

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let client_stream = std::net::TcpStream::connect(addr).unwrap();
    let (server_stream, _) = listener.accept().unwrap();

    let client = WebSocket::from_raw_socket(MaybeTlsStream::Plain(client_stream), Role::Client, None);
    let server = WebSocket::from_raw_socket(MaybeTlsStream::Plain(server_stream), Role::Server, None);
    (client, server)
}

use serde_json::json;

#[test]
fn handle_bwin_event_replies_to_ping_on_the_socket() {
    use tokio_tungstenite::tungstenite::Message;

    let (mut client_ws, mut server_ws) = ws_pair();
    let (tx, _rx) = tokio::sync::mpsc::channel(10);

    BwinConnector::handle_bwin_event(BwinWSEvent::Ping, &mut server_ws, &tx);

    // the connector answers pings with a signalr type-6 frame
    let reply = client_ws.read().unwrap();
    assert_eq!(
        Message::Text("{\"type\":6}\x1e".into()),
        reply
    );
}

#[test]
fn handle_bwin_event_ignores_informational_frames() {
    let (_client_ws, mut server_ws) = ws_pair();
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    let events = vec![
        BwinWSEvent::MainToLiveUpdate {
            switched_fixtures: vec![],
        },
        BwinWSEvent::FixtureUpdate {
            fixture_id: "f1".to_string(),
            stage: "Live".to_string(),
        },
        BwinWSEvent::ScoreboardSlim {
            scoreboard: json!({}),
            fixture_id: "f1".to_string(),
        },
        BwinWSEvent::ConnectionAck {
            connection_id: "conn-1".to_string(),
        },
        BwinWSEvent::Subscribe {
            topics: vec!["t1".to_string()],
        },
        BwinWSEvent::Close {
            error: "boom".to_string(),
            allow_reconnect: true,
        },
    ];

    for event in events {
        BwinConnector::handle_bwin_event(event, &mut server_ws, &tx);
    }

    assert!(rx.try_recv().is_err());
}

#[test]
fn handle_bwin_event_forwards_parsed_markets() {
    use crate::infrastructure::parsers::bwin_parser::SwitchedFixture;

    let (_client_ws, mut server_ws) = ws_pair();
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    // an OptionMarketUpdate with a valid 3way market must reach the channel
    let payload = json!({
        "optionMarket": {
            "id": 1,
            "status": "Visible",
            "parameters": [
                {"key": "Period", "value": "RegularTime"},
                {"key": "MarketType", "value": "3way"}
            ],
            "options": [
                {"price": {"odds": 2.0}},
                {"price": {"odds": 3.0}},
                {"price": {"odds": 4.0}}
            ]
        }
    });

    BwinConnector::handle_bwin_event(
        BwinWSEvent::OptionMarketUpdate {
            fixture_id: "f1".to_string(),
            payload,
        },
        &mut server_ws,
        &tx,
    );

    match rx.try_recv().unwrap() {
        BookmakerEvent::UpdateMarkets((fixture_id, markets)) => {
            assert_eq!("f1", fixture_id);
            assert_eq!(1, markets.len());
        }
        _ => panic!("expected update markets event"),
    }

    // a MainToLiveUpdate prints the switched fixtures without touching the channel
    BwinConnector::handle_bwin_event(
        BwinWSEvent::MainToLiveUpdate {
            switched_fixtures: vec![SwitchedFixture {
                pre_match_id: "1".to_string(),
                in_play_id: "2".to_string(),
            }],
        },
        &mut server_ws,
        &tx,
    );
    assert!(rx.try_recv().is_err());
}

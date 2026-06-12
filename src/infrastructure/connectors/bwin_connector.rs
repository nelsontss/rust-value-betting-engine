use std::net::TcpStream;

use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::connect;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::stream::MaybeTlsStream;
use tokio_tungstenite::tungstenite::{Message, WebSocket};

use crate::application::services::bookmaker_scrapper_service::{BookmakerEvent, Connector};
use crate::domain::{Game, Platform};
use crate::infrastructure::parsers::bwin_parser::{BwinParser, BwinWSEvent};
use crate::infrastructure::parsers::parser_registry::ParserRegistry;
use crate::shared::error::Result;

pub struct BwinConnector {}

impl Connector for BwinConnector {
    fn start(&self, sender: Sender<BookmakerEvent>) -> Result<()> {
        let registry = ParserRegistry::new();
        match BwinConnector::client()
            .get(BwinConnector::FIXTURES_URL)
            .send()
        {
            Ok(response) => {
                if let Ok(json) = response.json::<serde_json::Value>() {
                    match registry.parse(&Platform::Bwin, json) {
                        Some(games) => {
                            BwinConnector::subscribe_to_game_updates(games, sender);
                        }
                        None => {
                            eprintln!("no parser registered for platform LeBull");
                        }
                    }
                } else {
                    eprintln!("Error reading body json");
                }
            }
            Err(e) => {
                eprintln!("Error making polling request to bwin: {:?}", e);
            }
        }

        Ok(())
    }
}

impl BwinConnector {
    const FIXTURES_URL: &str = "https://www.bwin.pt/cds-api/bettingoffer/fixtures?x-bwin-accessid=YmQwNTFkNDAtNzM3Yi00YWIyLThkNDYtYWFmNGY2N2Y1OWIx&lang=en&country=PT&userCountry=PT&fixtureTypes=Standard&state=Latest&offerMapping=Filtered&offerCategories=Gridable&fixtureCategories=Gridable,NonGridable,Other&sportIds=4&isPriceBoost=false&statisticsModes=None&skip=0&take=50&sortBy=Tags";
    const WEBSOCKET_URL: &str = "wss://cds-push.bwin.pt/ws-1-0?lang=pt&country=PT&x-bwin-accessId=YmQwNTFkNDAtNzM3Yi00YWIyLThkNDYtYWFmNGY2N2Y1OWIx&appUpdates=false";

    pub fn client() -> reqwest::blocking::Client {
        reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36")
            .default_headers(Self::headers())
            .build()
            .expect("failed to build reqwest client")
    }

    pub fn headers() -> reqwest::header::HeaderMap {
        use reqwest::header::HeaderMap;
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "https://www.bwin.pt".parse().unwrap());
        headers.insert(
            "sec-ch-ua",
            "\"Chromium\";v=\"148\", \"Google Chrome\";v=\"148\", \"Not/A)Brand\";v=\"99\""
                .parse()
                .unwrap(),
        );
        headers.insert("sec-ch-ua-platform", "\"macOS\"".parse().unwrap());
        headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
        headers.insert(
            "Referer",
            "https://www.bwin.pt/pt/sports/futebol-4/hoje"
                .parse()
                .unwrap(),
        );
        headers.insert(
            "x-bwin-browser-url",
            "https://www.bwin.pt/pt/sports/futebol-4/hoje"
                .parse()
                .unwrap(),
        );
        headers.insert("X-From-Product", "host-app".parse().unwrap());
        headers.insert("X-Device-Type", "desktop_OS X".parse().unwrap());
        headers.insert(
            "Accept",
            "application/json, text/plain, */*".parse().unwrap(),
        );
        headers
    }

    fn subscribe_to_game_updates(games: Vec<Game>, sender: Sender<BookmakerEvent>) {
        let topics = BwinConnector::get_subcription_topics(&games);
        let _ = sender.blocking_send(BookmakerEvent::InsertGames(games));

        let url = BwinConnector::WEBSOCKET_URL;
        let mut request = url.into_client_request().unwrap();
        request
            .headers_mut()
            .insert("Origin", HeaderValue::from_static("https://www.bwin.pt"));
        request.headers_mut().insert(
            "User-Agent",
            HeaderValue::from_static(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36",
            ),
        );
        match connect(request) {
            Ok((mut ws, _)) => {
                ws.send(Message::Text(
                    "{\"protocol\":\"json\",\"version\":1}\x1e".into(),
                ))
                .unwrap();

                let frame = BwinWSEvent::subscribe(topics);
                let subscribe_msg = serde_json::to_string(&frame).unwrap() + "\x1e";
                let mut subscribed = false;

                while let Ok(message) = ws.read() {
                    match message {
                        Message::Text(text) => {
                            for part in text.split('\x1e') {
                                let part = part.trim();
                                if part.is_empty() {
                                    continue;
                                }
                                if part == "{}" && !subscribed {
                                    ws.send(Message::Text(subscribe_msg.clone())).unwrap();
                                    subscribed = true;
                                    continue;
                                }
                                if let Some(event) = BwinParser::parse_ws_event(part) {
                                    BwinConnector::handle_bwin_event(event, &mut ws, &sender);
                                }
                            }
                        }
                        Message::Close(_) => {
                            eprintln!("Bwin socket closed");
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("Error connecting to websocket: {:?}", e);
            }
        }
    }

    fn get_subcription_topics(games: &[Game]) -> Vec<String> {
        games
            .iter()
            .map(|g| format!("v2|pt|{}_67_any|grd", g.id))
            .collect()
    }

    fn handle_bwin_event(
        event: BwinWSEvent,
        ws: &mut WebSocket<MaybeTlsStream<TcpStream>>,
        sender: &Sender<BookmakerEvent>,
    ) {
        match event {
            BwinWSEvent::Ping => {
                ws.send(Message::Text("{\"type\":6}\x1e".into())).ok();
            }
            BwinWSEvent::OptionMarketUpdate {
                fixture_id,
                payload,
                ..
            } => {
                let markets = BwinParser::parse_option_market_update(payload);
                let _ = sender.blocking_send(BookmakerEvent::UpdateMarkets((fixture_id, markets)));
            }
            BwinWSEvent::MainToLiveUpdate { switched_fixtures } => {
                println!("switched: {:?}", switched_fixtures);
            }
            _ => {
                eprintln!("Unhandled bwin event")
            }
        }
    }

    pub fn new() -> Self {
        BwinConnector {}
    }
}

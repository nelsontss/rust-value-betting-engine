use std::net::TcpStream;
use std::time::Duration;

use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::connect;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::stream::MaybeTlsStream;
use tokio_tungstenite::tungstenite::{Message, WebSocket};

use crate::application::services::bookmaker_scrapper_service::BookmakerEvent;
use crate::domain::{Game, Platform};
use crate::infrastructure::parsers::bwin_parser::{BwinParser, BwinWSEvent};
use crate::infrastructure::parsers::parser_registry::ParserRegistry;
use crate::shared::error::Result;

pub struct BwinConnector {}

#[cfg(test)]
mod tests;

impl BwinConnector {
    pub async fn start(&self, sender: Sender<BookmakerEvent>) -> Result<()> {
        let registry = ParserRegistry::new();

        tokio::task::spawn_blocking(move || {
            Self::run_blocking(sender, registry);
        })
        .await
        .map_err(|e| format!("Bwin connector task failed: {e}"))?;

        Ok(())
    }

    fn run_blocking(sender: Sender<BookmakerEvent>, registry: ParserRegistry) {
        let mut backoff = Duration::from_secs(5);
        let max_backoff = Duration::from_secs(60);

        loop {
            match Self::fetch_and_subscribe(&registry, &sender) {
                Ok(()) => {
                    eprintln!("Bwin WS disconnected, reconnecting...");
                    backoff = Duration::from_secs(5);
                }
                Err(e) => {
                    eprintln!("Bwin connector error: {:?}, retrying in {:?}", e, backoff);
                }
            }

            std::thread::sleep(backoff);
            backoff = (backoff * 2).min(max_backoff);
        }
    }

    fn fetch_and_subscribe(
        registry: &ParserRegistry,
        sender: &Sender<BookmakerEvent>,
    ) -> Result<()> {
        let mut all_games = Vec::new();

        for url in [
            BwinConnector::FIXTURES_URL,
            BwinConnector::LIVE_FIXTURES_URL,
        ] {
            let response = BwinConnector::client()
                .get(url)
                .send()
                .map_err(|e| format!("Error fetching fixtures: {e}"))?;

            let json: serde_json::Value = response
                .json()
                .map_err(|e| format!("Error reading fixtures JSON: {e}"))?;

            if let Some(games) = registry.parse(&Platform::Bwin, json) {
                all_games.extend(games);
            }
        }

        if all_games.is_empty() {
            return Err("No games found".into());
        }

        let _ = sender.blocking_send(BookmakerEvent::InsertGames(all_games.clone()));
        Self::run_ws_session(all_games, sender)
    }

    fn run_ws_session(games: Vec<Game>, sender: &Sender<BookmakerEvent>) -> Result<()> {
        let topics = Self::get_subscription_topics(&games);

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

        let (mut ws, _) = connect(request).map_err(|e| format!("WS connect error: {e}"))?;

        ws.send(Message::Text(
            "{\"protocol\":\"json\",\"version\":1}\x1e".into(),
        ))
        .map_err(|e| format!("WS handshake error: {e}"))?;

        let mut subscribed = false;
        let subscribe_chunks: Vec<Vec<String>> = topics.chunks(40).map(|c| c.to_vec()).collect();
        let subscribe_msgs: Vec<String> = subscribe_chunks
            .into_iter()
            .map(|chunk| serde_json::to_string(&BwinWSEvent::subscribe(chunk)).unwrap() + "\x1e")
            .collect();

        while let Ok(message) = ws.read() {
            match message {
                Message::Text(text) => {
                    for part in text.split('\x1e') {
                        let part = part.trim();
                        if part.is_empty() {
                            continue;
                        }
                        if part == "{}" && !subscribed {
                            for msg in &subscribe_msgs {
                                ws.send(Message::Text(msg.clone().into())).ok();
                            }
                            subscribed = true;
                            continue;
                        }
                        if let Some(event) = BwinParser::parse_ws_event(part) {
                            Self::handle_bwin_event(event, &mut ws, &sender);
                        } else {
                            eprintln!("[bwin] unparsable frame: {}", &part[..part.len().min(200)]);
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

        Ok(())
    }

    fn get_subscription_topics(games: &[Game]) -> Vec<String> {
        games
            .iter()
            .flat_map(|g| {
                let base = format!("v2|pt|{}_67_any", g.id);
                let mut topics = vec![format!("{}|fxt", base), format!("{}|sbs", base)];
                for (_, market) in g.markets() {
                    let market_id = Self::market_bwin_id(market);
                    topics.push(format!("{}|fxm-{}", base, market_id));
                }
                topics
            })
            .collect()
    }

    fn market_bwin_id(market: &crate::domain::entities::Market) -> String {
        match market {
            crate::domain::entities::Market::MatchResult(m) => m.id.clone(),
            crate::domain::entities::Market::Moneyline(m) => m.id(),
            crate::domain::entities::Market::DoubleChance(m) => m.id(),
            crate::domain::entities::Market::Total(m) => m.id(),
            crate::domain::entities::Market::Handicap(m) => m.id(),
            crate::domain::entities::Market::AsianHandicap(m) => m.id(),
        }
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
                let markets = BwinParser::parse_option_market_update(payload.clone());

                if !markets.is_empty() {
                    let _ =
                        sender.blocking_send(BookmakerEvent::UpdateMarkets((fixture_id, markets)));
                }
            }
            BwinWSEvent::MainToLiveUpdate { switched_fixtures } => {
                for sf in &switched_fixtures {
                    println!(
                        "bwin fixture switched prematch {} -> live {}",
                        sf.pre_match_id, sf.in_play_id
                    );
                }
            }
            BwinWSEvent::FixtureUpdate { fixture_id, stage } => {
                println!("bwin fixture {} stage: {}", fixture_id, stage);
            }
            BwinWSEvent::OptionMarketDelete {
                market_id,
                fixture_id,
            } => {
                println!(
                    "bwin market deleted: fixture {} market {}",
                    fixture_id, market_id
                );
            }
            BwinWSEvent::ScoreboardSlim { .. } => {}
            BwinWSEvent::ConnectionAck { connection_id } => {
                println!("bwin connected: {}", connection_id);
            }
            BwinWSEvent::Subscribe { .. } => {}
            BwinWSEvent::Close { error, allow_reconnect } => {
                eprintln!("[bwin] close frame: {} allowReconnect={}", error, allow_reconnect);
            }
        }
    }

    const FIXTURES_URL: &str = "https://www.bwin.pt/cds-api/bettingoffer/fixtures?x-bwin-accessid=YmQwNTFkNDAtNzM3Yi00YWIyLThkNDYtYWFmNGY2N2Y1OWIx&lang=en&country=PT&userCountry=PT&fixtureTypes=Standard&state=Latest&offerMapping=Filtered&offerCategories=Gridable&fixtureCategories=Gridable,NonGridable,Other&sportIds=4&isPriceBoost=false&statisticsModes=None&skip=0&take=50&sortBy=Tags";
    const LIVE_FIXTURES_URL: &str = "https://www.bwin.pt/cds-api/bettingoffer/fixtures?x-bwin-accessid=YmQwNTFkNDAtNzM3Yi00YWIyLThkNDYtYWFmNGY2N2Y1OWIx&lang=en&country=PT&userCountry=PT&fixtureTypes=Standard&state=Live&offerMapping=Filtered&offerCategories=Gridable&fixtureCategories=Gridable,NonGridable,Other&sportIds=4&isPriceBoost=false&statisticsModes=None&skip=0&take=50&sortBy=Tags";
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

    pub fn new() -> Self {
        BwinConnector {}
    }
}

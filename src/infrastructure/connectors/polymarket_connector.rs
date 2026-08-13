use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use alloy::primitives::U256;
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use polymarket_client_sdk_v2::gamma::{
    Client,
    types::{
        request::EventsRequest,
        response::{Event, Market},
    },
};
use rust_decimal::prelude::*;
use tokio::{net::TcpStream, sync::mpsc::Sender};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, info, warn};

use crate::{
    application::services::bookmaker_scrapper_service::BookmakerEvent,
    domain::{
        self, Game, Platform,
        entities::{
            Odd,
            markets::{
                Line, asian_handicap::AsianHandicapMarket, double_chance::DoubleChanceMarket,
                match_result::MatchResultMarket, total::TotalMarket,
            },
        },
    },
    shared::error::Result,
};

pub struct PolymarketConnector {
    gamma_client: Client,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GammaSport {
    sport: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GammaEvent {
    #[serde(flatten)]
    event: Event,
    #[serde(rename = "sport")]
    sport: Option<GammaSport>,
}

type PriceWebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Deserialize)]
struct WireFrameEnvelope {
    #[serde(rename = "event_type")]
    event_type: String,
}

#[derive(Debug, Deserialize)]
struct WirePriceChange {
    #[serde(rename = "price_changes", default)]
    price_changes: Vec<WirePriceEntry>,
}

#[derive(Debug, Deserialize)]
struct WirePriceEntry {
    #[serde(rename = "asset_id")]
    asset_id: U256,
    price: Decimal,
}

#[derive(Debug, Deserialize)]
struct WireNewMarket {
    id: String,
    #[serde(rename = "assets_ids", alias = "asset_ids", default)]
    asset_ids: Vec<U256>,
    #[serde(rename = "event_message", default)]
    event_message: Option<WireEventMessage>,
}

#[derive(Debug, Deserialize)]
struct WireEventMessage {
    #[serde(default)]
    id: Option<String>,
}

const LEAGUE_CODE_TO_COUNTRY: &[(&str, &str)] = &[
    ("epl", "England"),
    ("efa", "England"),
    ("efl", "England"),
    ("elc", "England"),
    ("el1", "England"),
    ("el2", "England"),
    ("enl", "England"),
    ("ecs", "England"),
    ("scop", "Scotland"),
    ("sclc", "Scotland"),
    ("scoc", "Scotland"),
    ("ire", "Ireland"),
    ("irl1", "Ireland"),
    ("nirl1", "Northern Ireland"),
    ("bun", "Germany"),
    ("bl2", "Germany"),
    ("dfb", "Germany"),
    ("lal", "Spain"),
    ("es2", "Spain"),
    ("cdr", "Spain"),
    ("ssc", "Spain"),
    ("sea", "Italy"),
    ("itsb", "Italy"),
    ("itc", "Italy"),
    ("isc", "Italy"),
    ("fl1", "France"),
    ("fr2", "France"),
    ("cde", "France"),
    ("frtc", "France"),
    ("ere", "Netherlands"),
    ("ned2", "Netherlands"),
    ("nlc", "Netherlands"),
    ("por", "Portugal"),
    ("ptc", "Portugal"),
    ("ptsc", "Portugal"),
    ("tur", "Turkey"),
    ("trsk", "Turkey"),
    ("tur2", "Turkey"),
    ("mls", "USA"),
    ("usl1", "USA"),
    ("uslc", "USA"),
    ("usoc", "USA"),
    ("lec", "USA"),
    ("ccup", "USA"),
    ("canpl", "Canada"),
    ("mex", "Mexico"),
    ("chi1", "Chile"),
    ("chl2", "Chile"),
    ("arg", "Argentina"),
    ("argcopa", "Argentina"),
    ("argpn", "Argentina"),
    ("bra", "Brazil"),
    ("bra2", "Brazil"),
    ("bra3", "Brazil"),
    ("brcm", "Brazil"),
    ("brco", "Brazil"),
    ("col1", "Colombia"),
    ("col2", "Colombia"),
    ("per1", "Peru"),
    ("par1", "Paraguay"),
    ("uru1", "Uruguay"),
    ("ven1", "Venezuela"),
    ("bol1", "Bolivia"),
    ("ecu1", "Ecuador"),
    ("fpd", "Costa Rica"),
    ("gtm", "Guatemala"),
    ("swe", "Sweden"),
    ("swe2", "Sweden"),
    ("nor", "Norway"),
    ("nor2", "Norway"),
    ("den", "Denmark"),
    ("fin1", "Finland"),
    ("fro1", "Faroe Islands"),
    ("isl1", "Iceland"),
    ("ltu1", "Lithuania"),
    ("lva1", "Latvia"),
    ("est1", "Estonia"),
    ("pol", "Poland"),
    ("cze1", "Czechia"),
    ("svk1", "Slovakia"),
    ("hun", "Hungary"),
    ("ukr1", "Ukraine"),
    ("rus", "Russia"),
    ("blr1", "Belarus"),
    ("kaz1", "Kazakhstan"),
    ("uzb1", "Uzbekistan"),
    ("aze1", "Azerbaijan"),
    ("aze2", "Azerbaijan"),
    ("azec", "Azerbaijan"),
    ("geo1", "Georgia"),
    ("isr", "Israel"),
    ("qat1", "Qatar"),
    ("spl", "Saudi Arabia"),
    ("skc", "Saudi Arabia"),
    ("uae1", "UAE"),
    ("egy1", "Egypt"),
    ("mar1", "Morocco"),
    ("saf1", "South Africa"),
    ("chi", "China"),
    ("chi2", "China"),
    ("chfa", "China"),
    ("jap", "Japan"),
    ("j1100", "Japan"),
    ("j2100", "Japan"),
    ("ja2", "Japan"),
    ("kor", "South Korea"),
    ("kor2", "South Korea"),
    ("aus", "Australia"),
    ("auc", "Australia"),
    ("tpe1", "Taiwan"),
    ("tpew", "Taiwan"),
    ("isp", "India"),
    ("rou1", "Romania"),
    ("srb", "Serbia"),
    ("slo", "Slovenia"),
    ("hr1", "Croatia"),
    ("grc", "Greece"),
    ("gre1", "Greece"),
    ("bul", "Bulgaria"),
    ("sui", "Switzerland"),
    ("atc", "Austria"),
    ("aut", "Austria"),
    ("bel1", "Belgium"),
    ("bel2", "Belgium"),
    ("ucl", "Europe"),
    ("uel", "Europe"),
    ("col", "Europe"),
    ("euc", "Europe"),
    ("uef", "Europe"),
    ("ewq", "Europe"),
    ("ueq", "Europe"),
    ("unl", "Europe"),
    ("uwcl", "Europe"),
    ("weuc", "Europe"),
    ("wwcquefa", "Europe"),
    ("acle", "Asia"),
    ("afcl", "Asia"),
    ("afc", "Asia"),
    ("aswq", "Asia"),
    ("aseanc", "Asia"),
    ("asean", "Asia"),
    ("aseanw", "Asia"),
    ("acn", "Africa"),
    ("caf", "Africa"),
    ("cafcl", "Africa"),
    ("afwq", "Africa"),
    ("lib", "South America"),
    ("sud", "South America"),
    ("copaam", "South America"),
    ("con", "South America"),
    ("sawq", "South America"),
    ("ccc", "North America"),
    ("conl", "North America"),
    ("cof", "North America"),
    ("ncag", "North America"),
    ("owq", "Oceania"),
    ("ofc", "Oceania"),
    ("cwc", "International"),
    ("fifwc", "International"),
    ("fifaw", "International"),
    ("fif", "International"),
    ("clf", "International"),
    ("icwq", "International"),
];

fn resolve_country(event: &GammaEvent) -> String {
    if let Some(country) = event.event.country_name.as_deref() {
        if !country.is_empty() {
            return country.to_string();
        }
    }
    event
        .sport
        .as_ref()
        .and_then(|sport| {
            LEAGUE_CODE_TO_COUNTRY
                .iter()
                .find(|(code, _)| *code == sport.sport)
                .map(|(_, country)| *country)
        })
        .unwrap_or_default()
        .to_string()
}

impl PolymarketConnector {
    pub fn new() -> Self {
        PolymarketConnector {
            gamma_client: Client::default(),
        }
    }

    pub async fn start(&self, sender: Sender<BookmakerEvent>) -> Result<()> {
        let ws_url = "wss://ws-subscriptions-clob.polymarket.com/ws/market";
        let poll_interval = Duration::from_secs(3600);
        let retry_delay = Duration::from_secs(10);

        let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
        let mut token_to_event: HashMap<U256, String> = HashMap::new();
        let mut subscribed: HashSet<U256> = HashSet::new();

        Self::fetch_and_update(
            &self.gamma_client,
            &sender,
            &mut events_cache,
            &mut token_to_event,
        )
        .await;

        loop {
            match Self::run_price_stream(
                &self.gamma_client,
                &sender,
                &mut events_cache,
                &mut token_to_event,
                &mut subscribed,
                ws_url,
                poll_interval,
            )
            .await
            {
                Ok(()) => warn!("Polymarket price stream ended, reconnecting"),
                Err(e) => warn!(?e, "Polymarket price stream failed, reconnecting"),
            }

            subscribed.clear();
            tokio::time::sleep(retry_delay).await;
        }
    }

    async fn run_price_stream(
        gamma: &Client,
        sender: &Sender<BookmakerEvent>,
        events_cache: &mut HashMap<String, GammaEvent>,
        token_to_event: &mut HashMap<U256, String>,
        subscribed: &mut HashSet<U256>,
        ws_url: &str,
        poll_interval: Duration,
    ) -> Result<()> {
        let (mut ws, _) = connect_async(ws_url).await?;

        let token_ids: Vec<U256> = token_to_event.keys().copied().collect();
        Self::send_subscribe(&mut ws, &token_ids).await?;
        subscribed.extend(token_ids.iter().copied());
        info!(
            count = subscribed.len(),
            "Subscribed to Polymarket tokens on market channel"
        );

        let mut poll_timer = tokio::time::interval(poll_interval);
        poll_timer.tick().await;

        let mut ping_timer = tokio::time::interval(Duration::from_secs(10));
        ping_timer.tick().await;

        loop {
            tokio::select! {
                _ = poll_timer.tick() => {
                    Self::fetch_and_update(gamma, sender, events_cache, token_to_event).await;

                    let current: HashSet<U256> = token_to_event.keys().copied().collect();

                    let to_subscribe: Vec<U256> = current.difference(subscribed).copied().collect();
                    if !to_subscribe.is_empty() {
                        Self::send_subscribe(&mut ws, &to_subscribe).await?;
                        subscribed.extend(to_subscribe);
                    }

                    let to_unsubscribe: Vec<U256> = subscribed.difference(&current).copied().collect();
                    if !to_unsubscribe.is_empty() {
                        Self::send_unsubscribe(&mut ws, &to_unsubscribe).await?;
                        for token in &to_unsubscribe {
                            subscribed.remove(token);
                        }
                    }
                }
                _ = ping_timer.tick() => {
                    if let Err(e) = ws.send(Message::text("PING")).await {
                        warn!(?e, "Failed to send Polymarket WS PING");
                    }
                }
                frame = ws.next() => {
                    let Some(frame) = frame else {
                        warn!("Polymarket price stream ended");
                        return Ok(());
                    };
                    let frame = frame?;
                    match frame {
                        Message::Text(text) => {
                            Self::handle_frame(
                                gamma,
                                sender,
                                events_cache,
                                token_to_event,
                                subscribed,
                                &mut ws,
                                &text,
                            )
                            .await?;
                        }
                        Message::Ping(_) => {
                            let _ = ws.send(Message::Pong(vec![].into())).await;
                        }
                        Message::Pong(_) => {}
                        _ => {}
                    }
                }
            }
        }
    }

    async fn handle_frame(
        gamma: &Client,
        sender: &Sender<BookmakerEvent>,
        events_cache: &mut HashMap<String, GammaEvent>,
        token_to_event: &mut HashMap<U256, String>,
        subscribed: &mut HashSet<U256>,
        ws: &mut PriceWebSocket,
        text: &str,
    ) -> Result<()> {
        let envelope: WireFrameEnvelope = match serde_json::from_str(text) {
            Ok(e) => e,
            Err(e) => {
                warn!(?e, "Failed to parse Polymarket WS frame");
                return Ok(());
            }
        };

        match envelope.event_type.as_str() {
            "price_change" => {
                let price_change: WirePriceChange = match serde_json::from_str(text) {
                    Ok(pc) => pc,
                    Err(e) => {
                        warn!(?e, "Failed to parse price_change frame");
                        return Ok(());
                    }
                };
                Self::handle_price_update(sender, events_cache, token_to_event, price_change)
                    .await;
            }
            "new_market" => {
                let new_market: WireNewMarket = match serde_json::from_str(text) {
                    Ok(nm) => nm,
                    Err(e) => {
                        warn!(?e, "Failed to parse new_market frame");
                        return Ok(());
                    }
                };
                let new_tokens =
                    Self::handle_new_market(gamma, sender, events_cache, token_to_event, new_market)
                        .await;
                let new_tokens: Vec<U256> = new_tokens
                    .into_iter()
                    .filter(|token| !subscribed.contains(token))
                    .collect();
                if !new_tokens.is_empty() {
                    Self::send_subscribe(ws, &new_tokens).await?;
                    subscribed.extend(new_tokens);
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_new_market(
        gamma: &Client,
        sender: &Sender<BookmakerEvent>,
        events_cache: &mut HashMap<String, GammaEvent>,
        token_to_event: &mut HashMap<U256, String>,
        new_market: WireNewMarket,
    ) -> Vec<U256> {
        let Some(event_id) = new_market
            .event_message
            .as_ref()
            .and_then(|em| em.id.as_deref())
        else {
            debug!(
                market_id = %new_market.id,
                "new_market event without parent, ignoring"
            );
            return Vec::new();
        };

        if events_cache.contains_key(event_id) {
            let Ok(market) = Self::fetch_market_by_id(gamma, &new_market.id).await else {
                return Vec::new();
            };
            let Some(event) = events_cache.get_mut(event_id) else {
                return Vec::new();
            };
            if !upsert_event_market(event, market) {
                return Vec::new();
            }
            if let Some(game) = event_to_game(event) {
                let game_id = game.id.clone();
                let markets: Vec<_> = game.markets().values().cloned().collect();
                let _ = sender
                    .send(BookmakerEvent::UpdateMarkets((game_id, markets)))
                    .await;
            }
        } else {
            let Ok(event) = Self::fetch_event_by_id(gamma, event_id).await else {
                return Vec::new();
            };
            if !is_soccer_event(&event) {
                return Vec::new();
            }
            let event_id = event.event.id.clone();
            events_cache.insert(event_id.clone(), event);
            if let Some(game) = events_cache.get(&event_id).and_then(event_to_game) {
                let _ = sender.send(BookmakerEvent::InsertGames(vec![game])).await;
            }
        }

        let mut new_tokens = Vec::new();
        for token in new_market.asset_ids {
            token_to_event.insert(token, event_id.to_string());
            new_tokens.push(token);
        }
        new_tokens
    }

    async fn send_subscribe(ws: &mut PriceWebSocket, token_ids: &[U256]) -> Result<()> {
        if token_ids.is_empty() {
            return Ok(());
        }
        let asset_ids: Vec<String> = token_ids.iter().map(ToString::to_string).collect();
        let frame = serde_json::json!({
            "assets_ids": asset_ids,
            "type": "market",
            "custom_feature_enabled": true,
        });
        ws.send(Message::text(frame.to_string())).await?;
        debug!(count = token_ids.len(), "Subscribed to new Polymarket tokens");
        Ok(())
    }

    async fn send_unsubscribe(ws: &mut PriceWebSocket, token_ids: &[U256]) -> Result<()> {
        if token_ids.is_empty() {
            return Ok(());
        }
        let asset_ids: Vec<String> = token_ids.iter().map(ToString::to_string).collect();
        let frame = serde_json::json!({
            "assets_ids": asset_ids,
            "operation": "unsubscribe",
        });
        ws.send(Message::text(frame.to_string())).await?;
        debug!(count = token_ids.len(), "Unsubscribed from Polymarket tokens");
        Ok(())
    }

    async fn fetch_market_by_id(gamma: &Client, id: &str) -> Result<Market> {
        let host = gamma.host();
        let response = reqwest::get(format!("{host}markets/{id}")).await?;
        if !response.status().is_success() {
            warn!(status = %response.status(), market_id = id, "Failed to fetch market by id");
        }
        Ok(response.json().await?)
    }

    async fn fetch_event_by_id(gamma: &Client, id: &str) -> Result<GammaEvent> {
        let host = gamma.host();
        let response = reqwest::get(format!("{host}events/{id}")).await?;
        if !response.status().is_success() {
            warn!(status = %response.status(), event_id = id, "Failed to fetch event by id");
        }
        Ok(response.json().await?)
    }

    async fn fetch_and_update(
        gamma: &Client,
        sender: &Sender<BookmakerEvent>,
        events_cache: &mut HashMap<String, GammaEvent>,
        token_map: &mut HashMap<U256, String>,
    ) {
        let events = match Self::fetch_events(gamma).await {
            Ok(e) => e,
            Err(e) => {
                warn!(?e, "Failed to fetch Polymarket events");
                return;
            }
        };

        let mut games = Vec::new();
        let mut new_token_map: HashMap<U256, String> = HashMap::new();

        for event in &events {
            if let Some(game) = event_to_game(event) {
                if let Some(markets) = &event.event.markets {
                    for market in markets {
                        if let Some(token_ids) = &market.clob_token_ids {
                            if let Some(&yes_token) = token_ids.first() {
                                new_token_map.insert(yes_token, game.id.clone());
                            }
                        }
                    }
                }
                games.push(game);
            }
        }

        for event in events {
            events_cache.insert(event.event.id.clone(), event);
        }
        *token_map = new_token_map;

        if !games.is_empty() {
            let _ = sender.send(BookmakerEvent::InsertGames(games)).await;
        }
    }

    async fn fetch_events(gamma: &Client) -> Result<Vec<GammaEvent>> {
        let host = gamma.host();
        let end_date_min = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
        let end_date_max = (Utc::now() + Duration::from_hours(48)).format("%Y-%m-%dT%H:%M:%SZ");

        let mut events = Vec::new();
        let mut offset = 0;
        loop {
            let url = format!(
                "{host}events?tag_slug=soccer&closed=false&ascending=true&limit=100&offset={offset}&end_date_min={end_date_min}&end_date_max={end_date_max}"
            );
            let response = reqwest::get(&url).await?;
            if !response.status().is_success() {
                warn!(
                    status = %response.status(),
                    offset,
                    "Polymarket events page failed, stopping pagination"
                );
                break;
            }
            let page: Vec<GammaEvent> = response.json().await?;
            let page_len = page.len();
            events.extend(page);
            if page_len < 100 {
                break;
            }
            offset += 100;
            if offset >= 5000 {
                warn!("Polymarket events pagination hit cap of 5000");
                break;
            }
        }
        Ok(events)
    }

    async fn handle_price_update(
        sender: &Sender<BookmakerEvent>,
        events_cache: &mut HashMap<String, GammaEvent>,
        token_to_event: &HashMap<U256, String>,
        price_change: WirePriceChange,
    ) {
        debug!(
            changes = price_change.price_changes.len(),
            "Received Polymarket price update"
        );
        for entry in &price_change.price_changes {
            let token_id = entry.asset_id;
            let Some(event_id) = token_to_event.get(&token_id) else {
                continue;
            };
            let Some(mut event) = events_cache.get(event_id).cloned() else {
                continue;
            };

            let mut updated = false;
            if let Some(ref mut markets) = event.event.markets {
                for market in markets.iter_mut() {
                    if let Some(token_ids) = &market.clob_token_ids {
                        if let Some(pos) = token_ids.iter().position(|&t| t == token_id) {
                            if let Some(ref mut prices) = market.outcome_prices {
                                if pos < prices.len() {
                                    prices[pos] = entry.price;
                                    updated = true;
                                }
                            }
                        }
                    }
                }
            }

            if updated {
                events_cache.insert(event.event.id.clone(), event.clone());
                if let Some(game) = event_to_game(&event) {
                    let game_id = game.id.clone();
                    let markets: Vec<_> = game.markets().values().cloned().collect();
                    let _ = sender
                        .send(BookmakerEvent::UpdateMarkets((game_id, markets)))
                        .await;
                }
            }
        }
    }

    pub async fn get_historicall_football_markets(&self) -> Result<Vec<Event>> {
        let historical_football_events_request = EventsRequest::builder()
            .closed(true)
            .tag_slug("soccer".to_string())
            .limit(100)
            .build();

        let result = self
            .gamma_client
            .events(&historical_football_events_request)
            .await?;

        Ok(result)
    }
}

fn is_soccer_event(event: &GammaEvent) -> bool {
    let soccer = "soccer";
    if event.sport.as_ref().is_some_and(|s| s.sport == soccer) {
        return true;
    }
    event
        .event
        .tags
        .as_deref()
        .is_some_and(|tags| tags.iter().any(|t| t.slug.as_deref() == Some(soccer)))
}

fn upsert_event_market(event: &mut GammaEvent, market: Market) -> bool {
    let Some(markets) = event.event.markets.as_mut() else {
        event.event.markets = Some(vec![market]);
        return true;
    };
    if markets.iter().any(|m| m.id == market.id) {
        false
    } else {
        markets.push(market);
        true
    }
}

fn group_markets_by_type(markets: &[Market]) -> HashMap<String, Vec<&Market>> {
    markets.iter().fold(HashMap::new(), |mut acc, m| {
        let key = m.sports_market_type.clone().unwrap_or_default();
        acc.entry(key).or_default().push(m);
        acc
    })
}

fn event_to_game(event: &GammaEvent) -> Option<Game> {
    let (home_team, away_team) = parse_teams(event)?;

    let country = resolve_country(event);

    let competition = event
        .event
        .series
        .as_ref()
        .and_then(|s| s.first())
        .and_then(|s| s.title.as_deref())
        .or(event.event.series_slug.as_deref());

    let Some(competition) = competition else {
        warn!(event_id = %event.event.id, "Missing series title and series_slug");
        return None;
    };

    let game_start = event.event.start_time.map(|t| t.naive_utc()).or_else(|| {
        let first_market = event.event.markets.as_ref().and_then(|m| m.first())?;
        let game_start_str = first_market.game_start_time.as_deref()?;
        chrono::DateTime::parse_from_str(game_start_str, "%Y-%m-%d %H:%M:%S%z")
            .ok()
            .map(|t| t.with_timezone(&Utc).naive_utc())
    });

    let Some(game_start) = game_start else {
        warn!(event_id = %event.event.id, "Event has no start_time");
        return None;
    };

    let grouped = group_markets_by_type(event.event.markets.as_deref().unwrap_or_default());
    let game_markets = grouped_markets_to_game_markets(&home_team, &away_team, grouped);

    if game_markets.is_empty() {
        return None;
    }

    Some(Game::new_with_id(
        &event.event.id,
        &home_team,
        &away_team,
        &country,
        competition,
        game_start,
        Platform::Polymarket,
        game_markets,
    ))
}

fn parse_teams(event: &GammaEvent) -> Option<(String, String)> {
    if let (Some(home), Some(away)) = (
        event.event.home_team_name.as_deref(),
        event.event.away_team_name.as_deref(),
    ) {
        if !home.is_empty() && !away.is_empty() {
            return Some((home.to_string(), away.to_string()));
        }
    }

    let title = event.event.title.as_deref()?;
    let mut best: Option<(usize, usize)> = None;
    for sep in [" vs. ", " vs "] {
        if let Some(pos) = title.find(sep) {
            if best.map_or(true, |(p, _)| pos < p) {
                best = Some((pos, sep.len()));
            }
        }
    }
    let (pos, sep_len) = best?;

    let home = title[..pos].trim().split(" - ").next().unwrap_or_default();
    let away = title[pos + sep_len..]
        .trim()
        .split(" - ")
        .next()
        .unwrap_or_default();

    if home.is_empty() || away.is_empty() {
        return None;
    }

    Some((home.to_string(), away.to_string()))
}

fn prob_to_odd(prob: f64) -> Option<Odd> {
    if prob <= 0.0 || prob > 1.0 {
        return None;
    }
    let prob = Decimal::from_f64(prob)?;
    Odd::new_from_prob(prob).ok()
}

fn price_at(m: &Market, index: usize) -> Option<Odd> {
    prob_to_odd(m.outcome_prices.as_ref()?.get(index)?.to_f64()?)
}

fn market_line(m: &Market) -> f32 {
    m.line
        .and_then(|l| l.to_f64())
        .unwrap_or(0.0) as f32
}

fn classify_binary_market(
    title: &str,
    home_team: &str,
    away_team: &str,
) -> &'static str {
    let title = title.to_lowercase();
    let home = home_team.to_lowercase();
    let away = away_team.to_lowercase();

    if title.contains("draw") {
        "draw"
    } else if title.contains(&home) {
        "home"
    } else if title.contains(&away) {
        "away"
    } else {
        "unknown"
    }
}

fn match_result_market(
    home_team: &str,
    away_team: &str,
    markets: &[&Market],
) -> Option<domain::Market> {
    let mut home: Option<Odd> = None;
    let mut draw: Option<Odd> = None;
    let mut away: Option<Odd> = None;

    for market in markets {
        let title = market.group_item_title.as_deref()?;
        let odd = price_at(market, 0)?;
        match classify_binary_market(title, home_team, away_team) {
            "home" => home = Some(odd),
            "draw" => draw = Some(odd),
            "away" => away = Some(odd),
            _ => (),
        }
    }

    let id = &markets.first()?.id;
    Some(domain::Market::MatchResult(MatchResultMarket::new(
        id,
        home?,
        draw?,
        away?,
    )))
}

fn double_chance_market(markets: &[&Market]) -> Option<domain::Market> {
    let mut home_or_draw: Option<Odd> = None;
    let mut home_or_away: Option<Odd> = None;
    let mut draw_or_away: Option<Odd> = None;

    for market in markets {
        let title = market.group_item_title.as_deref()?.to_lowercase();
        let odd = price_at(market, 0)?;
        if title.contains("1x") || (title.contains("home") && title.contains("draw")) {
            home_or_draw = Some(odd);
        } else if title.contains("12") || (title.contains("home") && title.contains("away")) {
            home_or_away = Some(odd);
        } else if title.contains("x2") || (title.contains("draw") && title.contains("away")) {
            draw_or_away = Some(odd);
        }
    }

    Some(domain::Market::DoubleChance(DoubleChanceMarket::new(
        markets.first()?.id.clone(),
        home_or_draw?,
        home_or_away?,
        draw_or_away?,
    )))
}

fn total_market(market: &Market) -> Option<domain::Market> {
    Some(domain::Market::Total(TotalMarket::new(
        market.id.clone(),
        Line(market_line(market)),
        price_at(market, 0)?,
        price_at(market, 1)?,
    )))
}

fn asian_handicap_market(
    home_team: &str,
    away_team: &str,
    market: &Market,
) -> Option<domain::Market> {
    let title = market
        .group_item_title
        .as_deref()
        .or(market.question.as_deref())
        .unwrap_or_default();
    let line = market_line(market);

    match classify_binary_market(title, home_team, away_team) {
        "home" => Some(domain::Market::AsianHandicap(AsianHandicapMarket::new(
            market.id.clone(),
            Line(line),
            price_at(market, 0)?,
            price_at(market, 1)?,
        ))),
        "away" => Some(domain::Market::AsianHandicap(AsianHandicapMarket::new(
            market.id.clone(),
            Line(-line),
            price_at(market, 1)?,
            price_at(market, 0)?,
        ))),
        _ => None,
    }
}

fn grouped_markets_to_game_markets(
    home_team: &str,
    away_team: &str,
    grouped_markets: HashMap<String, Vec<&Market>>,
) -> Vec<domain::Market> {
    let mut result = Vec::new();

    for (market_type, markets) in grouped_markets {
        let game_markets: Vec<domain::Market> = match market_type.as_str() {
            "moneyline" | "match result" => {
                match_result_market(home_team, away_team, &markets)
                    .into_iter()
                    .collect()
            }
            "double chance" | "double_chance" => {
                double_chance_market(&markets).into_iter().collect()
            }
            "totals" => markets.iter().filter_map(|m| total_market(m)).collect(),
            "spreads" | "asian handicap" | "asian_handicap" => markets
                .iter()
                .filter_map(|m| asian_handicap_market(home_team, away_team, m))
                .collect(),
            _ => Vec::new(),
        };

        result.extend(game_markets);
    }

    result
}

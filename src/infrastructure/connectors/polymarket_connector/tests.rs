use super::*;
use chrono::TimeZone;
use serde_json::json;

fn market_from_json(value: serde_json::Value) -> Market {
    serde_json::from_value(value).unwrap()
}

fn refs(markets: &[Market]) -> Vec<&Market> {
    markets.iter().collect()
}

fn gamma_event_from_json(value: serde_json::Value) -> GammaEvent {
    serde_json::from_value(value).unwrap()
}

fn binary_market(id: &str, title: &str, yes: f64, no: f64) -> Market {
    market_from_json(json!({
        "id": id,
        "groupItemTitle": title,
        "outcomePrices": format!("[{}, {}]", yes, no),
    }))
}

fn typed_market(
    id: &str,
    title: &str,
    yes: f64,
    no: f64,
    sports_type: &str,
    line: Option<f64>,
) -> Market {
    market_from_json(json!({
        "id": id,
        "groupItemTitle": title,
        "outcomePrices": format!("[{}, {}]", yes, no),
        "sportsMarketType": sports_type,
        "line": line,
    }))
}

#[test]
fn prob_to_odd_builds_odd_with_no_derived_probability() {
    let odd = prob_to_odd(0.55, 0.45).unwrap();

    assert!((odd.get() - 1.0 / 0.55).abs() < 1e-9);
    // the derived-from-no probability is 1 - no_prob
    let derived = odd
        .get_implied_probability_derived_from_no()
        .unwrap()
        .to_f64()
        .unwrap();
    assert!((derived - 0.55).abs() < 1e-9);
}

#[test]
fn prob_to_odd_rejects_probabilities_outside_zero_and_one() {
    assert!(prob_to_odd(0.0, 0.5).is_none());
    assert!(prob_to_odd(-0.1, 0.5).is_none());
    assert!(prob_to_odd(1.5, 0.5).is_none());
}

#[test]
fn derive_display_price_prefers_best_ask_and_falls_back_to_last_price() {
    let (ask, price, bid) = (Decimal::new(42, 2), Decimal::new(41, 2), Decimal::new(40, 2));

    assert_eq!(ask, derive_display_price_from_levels(bid, ask, price));
    assert_eq!(price, derive_display_price_from_levels(bid, Decimal::ZERO, price));
}

#[test]
fn classify_binary_market_matches_teams_and_draw() {
    assert_eq!("home", classify_binary_market("Arsenal", "Arsenal", "Chelsea"));
    assert_eq!("away", classify_binary_market("Chelsea", "Arsenal", "Chelsea"));
    assert_eq!("draw", classify_binary_market("Draw", "Arsenal", "Chelsea"));
    assert_eq!("unknown", classify_binary_market("Over 2.5", "Arsenal", "Chelsea"));
}

#[test]
fn parse_teams_prefers_explicit_team_names_over_title() {
    let event = gamma_event_from_json(json!({
        "id": "e1",
        "title": "Arsenal vs. Chelsea",
        "homeTeamName": "Arsenal FC",
        "awayTeamName": "Chelsea FC",
    }));

    assert_eq!(
        Some(("Arsenal FC".to_string(), "Chelsea FC".to_string())),
        parse_teams(&event)
    );
}

#[test]
fn parse_teams_falls_back_to_title_with_vs_separator() {
    let event = gamma_event_from_json(json!({
        "id": "e1",
        "title": "Arsenal vs Chelsea - Premier League",
    }));

    assert_eq!(
        Some(("Arsenal".to_string(), "Chelsea".to_string())),
        parse_teams(&event)
    );
}

#[test]
fn parse_teams_handles_vs_dot_separator() {
    let event = gamma_event_from_json(json!({
        "id": "e1",
        "title": "Benfica vs. Sporting",
    }));

    assert_eq!(
        Some(("Benfica".to_string(), "Sporting".to_string())),
        parse_teams(&event)
    );
}

#[test]
fn parse_teams_returns_none_without_team_information() {
    let event = gamma_event_from_json(json!({ "id": "e1" }));

    assert_eq!(None, parse_teams(&event));
}

#[test]
fn polymarket_game_id_normalizes_and_dates_the_teams() {
    let date = NaiveDateTime::new(
        chrono::NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
        chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
    );

    let id = polymarket_game_id("Arsenal FC!", "  chelsea  ", date);

    assert_eq!("pm-arsenal_fc_vs_chelsea_20260510", id);
}

#[test]
fn is_soccer_event_detects_sport_field_and_soccer_tag() {
    let by_sport = gamma_event_from_json(json!({
        "id": "e1",
        "sport": { "sport": "soccer" },
    }));
    assert!(is_soccer_event(&by_sport));

    let by_tag = gamma_event_from_json(json!({
        "id": "e1",
        "tags": [{ "id": "t1", "slug": "soccer" }],
    }));
    assert!(is_soccer_event(&by_tag));

    let other = gamma_event_from_json(json!({
        "id": "e1",
        "sport": { "sport": "nba" },
        "tags": [{ "id": "t1", "slug": "basketball" }],
    }));
    assert!(!is_soccer_event(&other));
}

#[test]
fn is_event_expired_after_match_duration() {
    let past = gamma_event_from_json(json!({
        "id": "e1",
        "startTime": (Utc::now() - chrono::Duration::hours(3)).to_rfc3339(),
    }));
    assert!(is_event_expired(&past));

    let future = gamma_event_from_json(json!({
        "id": "e1",
        "startTime": (Utc::now() + chrono::Duration::hours(3)).to_rfc3339(),
    }));
    assert!(!is_event_expired(&future));

    let no_start = gamma_event_from_json(json!({ "id": "e1" }));
    assert!(!is_event_expired(&no_start));
}

#[test]
fn resolve_country_prefers_explicit_country_then_league_code() {
    let explicit = gamma_event_from_json(json!({
        "id": "e1",
        "countryName": "England",
        "sport": { "sport": "por" },
    }));
    assert_eq!("England", resolve_country(&explicit));

    let by_league = gamma_event_from_json(json!({
        "id": "e1",
        "sport": { "sport": "por" },
    }));
    assert_eq!("Portugal", resolve_country(&by_league));

    let unknown = gamma_event_from_json(json!({
        "id": "e1",
        "sport": { "sport": "made-up-league" },
    }));
    assert_eq!("", resolve_country(&unknown));
}

#[test]
fn upsert_event_market_appends_new_and_skips_duplicates() {
    let mut event = gamma_event_from_json(json!({ "id": "e1" }));
    let market = binary_market("m1", "Arsenal", 0.6, 0.4);

    assert!(upsert_event_market(&mut event, market.clone()));
    assert!(!upsert_event_market(&mut event, market));
    assert_eq!(1, event.event.markets.as_ref().unwrap().len());
}

#[test]
fn group_markets_by_type_buckets_on_sports_market_type() {
    let markets = vec![
        typed_market("m1", "Arsenal", 0.6, 0.4, "moneyline", None),
        typed_market("m2", "Draw", 0.2, 0.8, "moneyline", None),
        typed_market("m3", "Over 2.5", 0.5, 0.5, "totals", Some(2.5)),
    ];

    let grouped = group_markets_by_type(&markets);

    assert_eq!(2, grouped.len());
    assert_eq!(2, grouped["moneyline"].len());
    assert_eq!(1, grouped["totals"].len());
}

#[test]
fn market_line_reads_line_and_defaults_to_zero() {
    let with_line = market_from_json(json!({ "id": "m1", "line": 2.5 }));
    let without_line = market_from_json(json!({ "id": "m1" }));

    assert!((2.5 - market_line(&with_line)).abs() < 1e-6);
    assert!((0.0 - market_line(&without_line)).abs() < 1e-6);
}

#[test]
fn match_result_market_combines_home_draw_away_markets() {
    let markets = vec![
        typed_market("m1", "Arsenal", 0.55, 0.45, "moneyline", None),
        typed_market("m2", "Draw", 0.25, 0.75, "moneyline", None),
        typed_market("m3", "Chelsea", 0.20, 0.80, "moneyline", None),
    ];

    let market = match_result_market("Arsenal", "Chelsea", &refs(&markets)).unwrap();

    match market {
        domain::Market::MatchResult(m) => {
            assert_eq!("m1", m.id);
            assert!((m.home.get() - 1.0 / 0.55).abs() < 1e-9);
            assert!((m.draw.get() - 4.0).abs() < 1e-9);
            assert!((m.away.get() - 5.0).abs() < 1e-9);
        }
        other => panic!("expected match result market, got {:?}", other),
    }
}

#[test]
fn match_result_market_requires_all_three_outcomes() {
    let markets = vec![
        typed_market("m1", "Arsenal", 0.55, 0.45, "moneyline", None),
        typed_market("m2", "Draw", 0.25, 0.75, "moneyline", None),
    ];

    assert!(match_result_market("Arsenal", "Chelsea", &refs(&markets)).is_none());
}

#[test]
fn double_chance_market_combines_the_three_combinations() {
    let markets = vec![
        typed_market("m1", "Home or Draw", 0.75, 0.25, "double chance", None),
        typed_market("m2", "Home or Away", 0.85, 0.15, "double chance", None),
        typed_market("m3", "Draw or Away", 0.55, 0.45, "double chance", None),
    ];

    let market = double_chance_market(&refs(&markets)).unwrap();

    match market {
        domain::Market::DoubleChance(m) => {
            assert!((m.home_or_draw.get() - 1.0 / 0.75).abs() < 1e-9);
            assert!((m.home_or_away.get() - 1.0 / 0.85).abs() < 1e-9);
            assert!((m.draw_or_away.get() - 1.0 / 0.55).abs() < 1e-9);
        }
        other => panic!("expected double chance market, got {:?}", other),
    }
}

#[test]
fn total_market_uses_line_and_both_sides_of_the_book() {
    let market = typed_market("m1", "Over 2.5", 0.55, 0.45, "totals", Some(2.5));

    let total = total_market(&market).unwrap();

    match total {
        domain::Market::Total(m) => {
            assert!((m.line.0 - 2.5).abs() < 1e-6);
            assert!((m.over.get() - 1.0 / 0.55).abs() < 1e-9);
            assert!((m.under.get() - 1.0 / 0.45).abs() < 1e-9);
        }
        other => panic!("expected total market, got {:?}", other),
    }
}

#[test]
fn asian_handicap_market_negates_the_line_for_the_away_side() {
    let home = typed_market("m1", "Arsenal -1.5", 0.60, 0.40, "spreads", Some(-1.5));
    let away = typed_market("m2", "Chelsea +1.5", 0.40, 0.60, "spreads", Some(-1.5));

    match asian_handicap_market("Arsenal", "Chelsea", &home).unwrap() {
        domain::Market::AsianHandicap(m) => {
            assert!((m.line.0 - -1.5).abs() < 1e-6);
            assert!((m.home.get() - 1.0 / 0.60).abs() < 1e-9);
            assert!((m.away.get() - 1.0 / 0.40).abs() < 1e-9);
        }
        other => panic!("expected asian handicap market, got {:?}", other),
    }

    match asian_handicap_market("Arsenal", "Chelsea", &away).unwrap() {
        domain::Market::AsianHandicap(m) => {
            // the line flips sign for the away side
            assert!((m.line.0 - 1.5).abs() < 1e-6);
        }
        other => panic!("expected asian handicap market, got {:?}", other),
    }
}

#[test]
fn asian_handicap_market_skips_draw_titles() {
    let market = typed_market("m1", "Draw", 0.5, 0.5, "spreads", Some(-1.5));

    assert!(asian_handicap_market("Arsenal", "Chelsea", &market).is_none());
}

#[test]
fn event_to_game_builds_a_complete_game_from_gamma_payload() {
    let event = gamma_event_from_json(json!({
        "id": "e1",
        "slug": "epl-arsenal-vs-chelsea-2026-05-10",
        "homeTeamName": "Arsenal",
        "awayTeamName": "Chelsea",
        "startTime": "2026-05-10T18:00:00Z",
        "sport": { "sport": "epl" },
        "series": [{ "id": "s1", "title": "Premier League" }],
        "markets": [
            {
                "id": "m-home",
                "groupItemTitle": "Arsenal",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.55, 0.45]"
            },
            {
                "id": "m-draw",
                "groupItemTitle": "Draw",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.25, 0.75]"
            },
            {
                "id": "m-away",
                "groupItemTitle": "Chelsea",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.20, 0.80]"
            },
            {
                "id": "m-total",
                "groupItemTitle": "Over 2.5",
                "sportsMarketType": "totals",
                "line": 2.5,
                "outcomePrices": "[0.50, 0.50]"
            }
        ]
    }));

    let game = event_to_game(&event).unwrap();

    assert_eq!("pm-arsenal_vs_chelsea_20260510", game.id);
    assert_eq!("Arsenal", game.home_team());
    assert_eq!("Chelsea", game.away_team());
    assert_eq!("England", game.country());
    assert_eq!("Premier League", game.competition());
    assert_eq!(Platform::Polymarket, game.platform());
    assert_eq!(2, game.markets().len());
    // link keeps the slug up to and including the event date
    assert_eq!(
        Some("https://polymarket.com/sports/epl/epl-arsenal-vs-chelsea-2026-05-10"),
        game.link().map(|u| u.as_str())
    );
}

#[test]
fn event_to_game_returns_none_without_teams() {
    let event = gamma_event_from_json(json!({
        "id": "e1",
        "startTime": "2026-05-10T18:00:00Z",
        "series": [{ "id": "s1", "title": "Premier League" }],
    }));

    assert!(event_to_game(&event).is_none());
}

#[test]
fn event_to_game_returns_none_without_competition() {
    let event = gamma_event_from_json(json!({
        "id": "e1",
        "homeTeamName": "Arsenal",
        "awayTeamName": "Chelsea",
        "startTime": "2026-05-10T18:00:00Z",
    }));

    assert!(event_to_game(&event).is_none());
}

#[test]
fn event_to_game_returns_none_without_start_time() {
    let event = gamma_event_from_json(json!({
        "id": "e1",
        "homeTeamName": "Arsenal",
        "awayTeamName": "Chelsea",
        "series": [{ "id": "s1", "title": "Premier League" }],
    }));

    assert!(event_to_game(&event).is_none());
}

#[test]
fn event_to_game_falls_back_to_first_market_game_start_time() {
    let event = gamma_event_from_json(json!({
        "id": "e1",
        "homeTeamName": "Arsenal",
        "awayTeamName": "Chelsea",
        "series": [{ "id": "s1", "title": "Premier League" }],
        "markets": [
            {
                "id": "m-home",
                "groupItemTitle": "Arsenal",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.55, 0.45]",
                "gameStartTime": "2026-05-10 18:00:00 +0000"
            },
            {
                "id": "m-draw",
                "groupItemTitle": "Draw",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.25, 0.75]"
            },
            {
                "id": "m-away",
                "groupItemTitle": "Chelsea",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.20, 0.80]"
            }
        ]
    }));

    let game = event_to_game(&event).unwrap();

    assert_eq!(
        Utc.with_ymd_and_hms(2026, 5, 10, 18, 0, 0).unwrap(),
        game.date.and_utc()
    );
}

#[test]
fn event_to_game_returns_none_when_markets_produce_no_game_markets() {
    let event = gamma_event_from_json(json!({
        "id": "e1",
        "homeTeamName": "Arsenal",
        "awayTeamName": "Chelsea",
        "startTime": "2026-05-10T18:00:00Z",
        "series": [{ "id": "s1", "title": "Premier League" }],
        "markets": [
            {
                "id": "m-unknown",
                "groupItemTitle": "Something",
                "sportsMarketType": "exotic",
                "outcomePrices": "[0.50, 0.50]"
            }
        ]
    }));

    assert!(event_to_game(&event).is_none());
}

use crate::application::services::bookmaker_scrapper_service::BookmakerEvent;

struct BookmakerEventKind<'a>(&'a BookmakerEvent);
impl std::fmt::Debug for BookmakerEventKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            BookmakerEvent::InsertGames(games) => write!(f, "InsertGames({} games)", games.len()),
            BookmakerEvent::UpdateMarkets((id, markets)) => {
                write!(f, "UpdateMarkets({} -> {} markets)", id, markets.len())
            }
            BookmakerEvent::Error => write!(f, "Error"),
        }
    }
}

fn soccer_event_with_token(token: u64, yes: f64, no: f64) -> GammaEvent {
    gamma_event_from_json(json!({
        "id": "evt-1",
        "homeTeamName": "Arsenal",
        "awayTeamName": "Chelsea",
        "startTime": (Utc::now() + chrono::Duration::hours(2)).to_rfc3339(),
        "series": [{ "id": "s1", "title": "Premier League" }],
        "sport": { "sport": "epl" },
        "markets": [
            {
                "id": "m-home",
                "groupItemTitle": "Arsenal",
                "sportsMarketType": "moneyline",
                "outcomePrices": format!("[{}, {}]", yes, no),
                "clobTokenIds": format!("[{}]", token)
            },
            {
                "id": "m-draw",
                "groupItemTitle": "Draw",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.25, 0.75]",
                "clobTokenIds": "[778]"
            },
            {
                "id": "m-away",
                "groupItemTitle": "Chelsea",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.20, 0.80]",
                "clobTokenIds": "[779]"
            }
        ]
    }))
}

fn price_change(token: u64, price: f64) -> WirePriceChange {
    WirePriceChange {
        price_changes: vec![WirePriceEntry {
            asset_id: alloy::primitives::U256::from(token),
            price: Decimal::from_f64_retain(price).unwrap(),
            best_bid: Decimal::from_f64_retain(price - 0.01).unwrap(),
            best_ask: Decimal::from_f64_retain(price).unwrap(),
        }],
    }
}

#[tokio::test]
async fn handle_price_update_updates_price_and_emits_market_update() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
    let mut token_to_event: HashMap<U256, String> = HashMap::new();

    let event = soccer_event_with_token(777, 0.50, 0.50);
    events_cache.insert("evt-1".to_string(), event.clone());
    token_to_event.insert(U256::from(777u64), "evt-1".to_string());

    PolymarketConnector::handle_price_update(
        &tx,
        &mut events_cache,
        &token_to_event,
        price_change(777, 0.63),
    )
    .await;

    match rx.recv().await.unwrap() {
        BookmakerEvent::UpdateMarkets((game_id, markets)) => {
            assert!(game_id.starts_with("pm-arsenal_vs_chelsea"), "unexpected id {game_id}");
            // the home outcome price must reflect the new display price
            let has_updated_odd = markets.iter().any(|m| match m {
                domain::Market::MatchResult(mr) => (mr.home.get() - 1.0 / 0.63).abs() < 1e-9,
                _ => false,
            });
            assert!(has_updated_odd, "expected updated home odd, got {markets:?}");
        }
        other => panic!("expected update markets event, got {:?}", BookmakerEventKind(&other)),
    }

    // cache is refreshed with the new price
    let cached = &events_cache["evt-1"];
    let price = cached.event.markets.as_ref().unwrap()[0]
        .outcome_prices
        .as_ref()
        .unwrap()[0]
        .to_f64()
        .unwrap();
    assert!((price - 0.63).abs() < 1e-9);
}

#[tokio::test]
async fn handle_price_update_ignores_unknown_tokens() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
    let token_to_event: HashMap<U256, String> = HashMap::new();

    events_cache.insert("evt-1".to_string(), soccer_event_with_token(777, 0.5, 0.5));

    PolymarketConnector::handle_price_update(
        &tx,
        &mut events_cache,
        &token_to_event,
        price_change(999, 0.63),
    )
    .await;

    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn handle_price_update_ignores_closed_events() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
    let mut token_to_event: HashMap<U256, String> = HashMap::new();

    let event = gamma_event_from_json(json!({
        "id": "evt-1",
        "homeTeamName": "Arsenal",
        "awayTeamName": "Chelsea",
        "startTime": (Utc::now() + chrono::Duration::hours(2)).to_rfc3339(),
        "series": [{ "id": "s1", "title": "Premier League" }],
        "closed": true,
        "markets": [{
            "id": "m-home",
            "groupItemTitle": "Arsenal",
            "sportsMarketType": "moneyline",
            "outcomePrices": "[0.50, 0.50]",
            "clobTokenIds": "[777]"
        }]
    }));
    events_cache.insert("evt-1".to_string(), event);
    token_to_event.insert(U256::from(777u64), "evt-1".to_string());

    PolymarketConnector::handle_price_update(
        &tx,
        &mut events_cache,
        &token_to_event,
        price_change(777, 0.63),
    )
    .await;

    assert!(rx.try_recv().is_err());
}

#[test]
fn derive_display_price_entry_prefers_best_ask() {
    let entry = WirePriceEntry {
        asset_id: U256::from(1u64),
        price: Decimal::from_f64_retain(0.50).unwrap(),
        best_bid: Decimal::from_f64_retain(0.48).unwrap(),
        best_ask: Decimal::from_f64_retain(0.52).unwrap(),
    };

    assert_eq!(
        Decimal::from_f64_retain(0.52).unwrap(),
        derive_display_price(&entry)
    );
}

#[tokio::test]
async fn handle_frame_ignores_pong_and_invalid_frames() {
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
    let mut token_to_event: HashMap<U256, String> = HashMap::new();
    let mut subscribed: HashSet<U256> = HashSet::new();

    for text in ["PONG", "  pong  ", "not-json", r#"{"event_type":"unknown"}"#] {
        PolymarketConnector::handle_frame(
            &unused_gamma(),
            &tx,
            &mut events_cache,
            &mut token_to_event,
            &mut subscribed,
            &mut unused_ws().await,
            text,
        )
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn handle_frame_processes_price_change_frames() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
    let mut token_to_event: HashMap<U256, String> = HashMap::new();
    let mut subscribed: HashSet<U256> = HashSet::new();

    events_cache.insert("evt-1".to_string(), soccer_event_with_token(777, 0.5, 0.5));
    token_to_event.insert(U256::from(777u64), "evt-1".to_string());

    let frame = json!({
        "event_type": "price_change",
        "price_changes": [
            {"asset_id": "777", "price": "0.63", "best_bid": "0.62", "best_ask": "0.63"}
        ]
    });

    PolymarketConnector::handle_frame(
        &unused_gamma(),
        &tx,
        &mut events_cache,
        &mut token_to_event,
        &mut subscribed,
        &mut unused_ws().await,
        &frame.to_string(),
    )
    .await
    .unwrap();

    let event = rx.try_recv().expect("expected an update markets event");
    match event {
        BookmakerEvent::UpdateMarkets((game_id, _)) => {
            assert!(game_id.starts_with("pm-arsenal_vs_chelsea"));
        }
        other => panic!("expected update markets event, got {:?}", BookmakerEventKind(&other)),
    }
}

#[tokio::test]
async fn handle_new_market_without_parent_event_is_ignored() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
    let mut token_to_event: HashMap<U256, String> = HashMap::new();

    let new_market = serde_json::from_value::<WireNewMarket>(json!({
        "id": "m-new",
        "assets_ids": ["42"]
    }))
    .unwrap();

    let tokens = PolymarketConnector::handle_new_market(
        &unused_gamma(),
        &tx,
        &mut events_cache,
        &mut token_to_event,
        new_market,
    )
    .await;

    assert!(tokens.is_empty());
    assert!(rx.try_recv().is_err());
}

// -- helpers that never touch the network --

fn unused_gamma() -> crate::infrastructure::connectors::polymarket_connector::Client {
    polymarket_client_sdk_v2::gamma::Client::default()
}

async fn unused_ws() -> PriceWebSocket {
    // handle_frame only writes to the socket on the new_market branch; the
    // tested paths never touch it, so a plain connected TCP stream suffices
    use tokio_tungstenite::tungstenite::protocol::Role;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let _server = listener.accept().await.unwrap();

    tokio_tungstenite::WebSocketStream::from_raw_socket(
        MaybeTlsStream::Plain(stream),
        Role::Client,
        None,
    )
    .await
}

#[tokio::test]
async fn send_subscribe_writes_a_market_subscription_frame() {
    use futures::StreamExt;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let client_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let (server_stream, _) = listener.accept().await.unwrap();

    let mut client_ws: PriceWebSocket =
        tokio_tungstenite::WebSocketStream::from_raw_socket(
            MaybeTlsStream::Plain(client_stream),
            tokio_tungstenite::tungstenite::protocol::Role::Client,
            None,
        )
        .await;
    let mut server_ws: PriceWebSocket =
        tokio_tungstenite::WebSocketStream::from_raw_socket(
            MaybeTlsStream::Plain(server_stream),
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;

    PolymarketConnector::send_subscribe(&mut client_ws, &[U256::from(42u64)])
        .await
        .unwrap();

    let frame = server_ws.next().await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
    assert_eq!("market", parsed["type"]);
    assert_eq!("42", parsed["assets_ids"][0]);

    // empty subscriptions are a no-op that still succeeds
    PolymarketConnector::send_subscribe(&mut client_ws, &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn send_unsubscribe_writes_an_unsubscription_frame() {
    use futures::StreamExt;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let client_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let (server_stream, _) = listener.accept().await.unwrap();

    let mut client_ws: PriceWebSocket =
        tokio_tungstenite::WebSocketStream::from_raw_socket(
            MaybeTlsStream::Plain(client_stream),
            tokio_tungstenite::tungstenite::protocol::Role::Client,
            None,
        )
        .await;
    let mut server_ws: PriceWebSocket =
        tokio_tungstenite::WebSocketStream::from_raw_socket(
            MaybeTlsStream::Plain(server_stream),
            tokio_tungstenite::tungstenite::protocol::Role::Server,
            None,
        )
        .await;

    PolymarketConnector::send_unsubscribe(&mut client_ws, &[U256::from(7u64)])
        .await
        .unwrap();

    let frame = server_ws.next().await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
    assert_eq!("unsubscribe", parsed["operation"]);
}

// --- gamma API fetch tests against a local mock server ---

use polymarket_client_sdk_v2::gamma::Client as GammaClient;

fn gamma_at(uri: &str) -> GammaClient {
    GammaClient::new(&format!("{}/", uri.trim_end_matches('/'))).unwrap()
}

fn gamma_event_of(value: serde_json::Value) -> GammaEvent {
    gamma_event_from_json(value)
}

fn full_soccer_event_json(id: &str, token: u64) -> serde_json::Value {
    json!({
        "id": id,
        "slug": "epl-arsenal-vs-chelsea-2026-05-10",
        "homeTeamName": "Arsenal",
        "awayTeamName": "Chelsea",
        "startTime": (Utc::now() + chrono::Duration::hours(2)).to_rfc3339(),
        "series": [{ "id": "s1", "title": "Premier League" }],
        "sport": { "sport": "epl" },
        "markets": [
            {
                "id": format!("m-home-{}", token),
                "groupItemTitle": "Arsenal",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.55, 0.45]",
                "clobTokenIds": format!("[{}]", token)
            },
            {
                "id": "m-draw",
                "groupItemTitle": "Draw",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.25, 0.75]"
            },
            {
                "id": "m-away",
                "groupItemTitle": "Chelsea",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.20, 0.80]"
            }
        ]
    })
}

async fn mock_keyset(body: serde_json::Value) -> (wiremock::MockServer, GammaClient) {
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, ResponseTemplate};

    let server = wiremock::MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex("events/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    let gamma = gamma_at(&server.uri());
    (server, gamma)
}

#[tokio::test]
async fn fetch_events_parses_keyset_pages() {
    let (_server, gamma) = mock_keyset(json!({
        "events": [full_soccer_event_json("evt-1", 777)],
        "nextCursor": null
    }))
    .await;

    let events = PolymarketConnector::fetch_events(&gamma).await.unwrap();

    assert_eq!(1, events.len());
    assert_eq!("evt-1", events[0].event.id);
}

#[tokio::test]
async fn fetch_events_gives_up_after_retries_with_an_empty_page() {
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, ResponseTemplate};

    let server = wiremock::MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex("events/keyset"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let gamma = gamma_at(&server.uri());

    // three attempts, all failing -> graceful degradation to an empty result
    let events = PolymarketConnector::fetch_events(&gamma).await.unwrap();
    assert!(events.is_empty());
}

#[tokio::test]
async fn fetch_and_update_emits_games_and_builds_token_map() {
    let (_server, gamma) = mock_keyset(json!({
        "events": [
            full_soccer_event_json("evt-1", 777),
            // expired events are filtered out
            {
                "id": "evt-2",
                "homeTeamName": "Ajax",
                "awayTeamName": "Feyenoord",
                "startTime": (Utc::now() - chrono::Duration::hours(5)).to_rfc3339(),
                "series": [{ "id": "s1", "title": "Eredivisie" }],
                "sport": { "sport": "ere" },
                "markets": [{
                    "id": "m-x",
                    "groupItemTitle": "Ajax",
                    "sportsMarketType": "moneyline",
                    "outcomePrices": "[0.5, 0.5]"
                }]
            }
        ],
        "nextCursor": null
    }))
    .await;

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
    // stale cache entries are dropped
    events_cache.insert("evt-stale".to_string(), gamma_event_of(full_soccer_event_json("evt-stale", 111)));
    let mut token_map: HashMap<U256, String> = HashMap::new();

    PolymarketConnector::fetch_and_update(&gamma, &tx, &mut events_cache, &mut token_map).await;

    match rx.recv().await.unwrap() {
        BookmakerEvent::InsertGames(games) => {
            assert_eq!(1, games.len());
            assert!(games[0].id.starts_with("pm-arsenal_vs_chelsea"));
        }
        other => panic!("expected insert games, got {:?}", BookmakerEventKind(&other)),
    }

    // cache and token map were refreshed
    assert!(events_cache.contains_key("evt-1"));
    assert!(!events_cache.contains_key("evt-stale"));
    assert_eq!(
        Some(&"evt-1".to_string()),
        token_map.get(&U256::from(777u64))
    );
    assert!(!token_map.contains_key(&U256::from(111u64)));

    // no other events were emitted
    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn fetch_and_update_emits_market_updates_for_known_games() {
    // two gamma events in one page that map to the same game id:
    // the first inserts the game, the second becomes a market update
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
    let mut token_map: HashMap<U256, String> = HashMap::new();

    let (_server, gamma) = mock_keyset(json!({
        "events": [
            full_soccer_event_json("evt-1", 777),
            full_soccer_event_json("evt-1", 888)
        ],
        "nextCursor": null
    }))
    .await;

    PolymarketConnector::fetch_and_update(&gamma, &tx, &mut events_cache, &mut token_map).await;

    // the duplicate emits its update while iterating, the unique game is
    // inserted in one batch at the end
    let first = rx.recv().await.unwrap();
    match first {
        BookmakerEvent::UpdateMarkets((game_id, _)) => {
            assert!(game_id.starts_with("pm-arsenal_vs_chelsea"));
        }
        _ => panic!("expected update markets event"),
    }

    let second = rx.recv().await.unwrap();
    assert!(matches!(second, BookmakerEvent::InsertGames(_)));
}

#[tokio::test]
async fn handle_new_market_merges_into_cached_event_and_subscribes() {
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, ResponseTemplate};

    let server = wiremock::MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex("markets/m-new"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "id": "m-new",
                "groupItemTitle": "Draw",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.25, 0.75]"
            })),
        )
        .mount(&server)
        .await;
    let gamma = gamma_at(&server.uri());

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
    events_cache.insert("evt-1".to_string(), soccer_event_with_token(777, 0.5, 0.5));
    let mut token_to_event: HashMap<U256, String> = HashMap::new();

    let new_market = serde_json::from_value::<WireNewMarket>(json!({
        "id": "m-new",
        "assets_ids": ["999"],
        "event_message": { "id": "evt-1" }
    }))
    .unwrap();

    let new_tokens = PolymarketConnector::handle_new_market(
        &gamma,
        &tx,
        &mut events_cache,
        &mut token_to_event,
        new_market,
    )
    .await;

    // the market's asset was registered for subscription
    assert_eq!(vec![U256::from(999u64)], new_tokens);
    assert_eq!(Some(&"evt-1".to_string()), token_to_event.get(&U256::from(999u64)));

    // the cached event gained the draw market -> game update emitted
    match rx.recv().await.unwrap() {
        BookmakerEvent::UpdateMarkets((game_id, markets)) => {
            assert!(game_id.starts_with("pm-arsenal_vs_chelsea"));
            // home, draw and away merge into a single match result market
            assert_eq!(1, markets.len());
        }
        other => panic!("expected update markets, got {:?}", BookmakerEventKind(&other)),
    }
}

#[tokio::test]
async fn handle_frame_subscribes_new_market_tokens() {
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, ResponseTemplate};

    let server = wiremock::MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex("markets/m-new"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "id": "m-new",
                "groupItemTitle": "Draw",
                "sportsMarketType": "moneyline",
                "outcomePrices": "[0.25, 0.75]"
            })),
        )
        .mount(&server)
        .await;
    let gamma = gamma_at(&server.uri());

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let mut events_cache: HashMap<String, GammaEvent> = HashMap::new();
    events_cache.insert("evt-1".to_string(), soccer_event_with_token(777, 0.5, 0.5));
    let mut token_to_event: HashMap<U256, String> = HashMap::new();
    let mut subscribed: HashSet<U256> = HashSet::new();

    let mut ws = unused_ws().await;
    let frame = json!({
        "event_type": "new_market",
        "id": "m-new",
        "assets_ids": ["999"],
        "event_message": { "id": "evt-1" }
    });

    PolymarketConnector::handle_frame(
        &gamma,
        &tx,
        &mut events_cache,
        &mut token_to_event,
        &mut subscribed,
        &mut ws,
        &frame.to_string(),
    )
    .await
    .unwrap();

    // the new token was marked as subscribed
    assert!(subscribed.contains(&U256::from(999u64)));
    // and the game update was broadcast
    let event = rx.recv().await.unwrap();
    assert!(matches!(event, BookmakerEvent::UpdateMarkets(_)));
}

#[tokio::test]
async fn historicall_football_markets_fetches_closed_soccer_events() {
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, ResponseTemplate};

    let server = wiremock::MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex("events$"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(vec![full_soccer_event_json("evt-1", 777)]),
        )
        .mount(&server)
        .await;

    let connector = PolymarketConnector {
        gamma_client: gamma_at(&server.uri()),
    };

    let events = connector.get_historicall_football_markets().await.unwrap();
    assert_eq!(1, events.len());
    assert_eq!("evt-1", events[0].id);
}

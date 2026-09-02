use std::sync::Arc;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde_json::{Value, json};

use crate::domain::entities::{
    FixtureCluster, Game, Market, MarketType, Odd, Outcome, Platform,
    markets::{Line, moneyline::MoneylineMarket, total::TotalMarket},
};

use super::{cluster_response::ClusterResponse, game_response::GameResponse, market_history_response::MarketHistoryResponse, market_response::MarketResponse, statistics_response::StatisticsUpdatedResponse};

fn fixture_date() -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        NaiveTime::from_hms_milli_opt(18, 0, 0, 0).unwrap(),
    )
}

fn game(platform: Platform) -> Game {
    game_with_id("g1", platform)
}

fn game_with_id(id: &str, platform: Platform) -> Game {
    Game::new_with_id(
        id,
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        fixture_date(),
        platform,
        vec![
            Market::Moneyline(MoneylineMarket::new(
                "ml-1".to_string(),
                Odd::new(2.0).unwrap(),
                Odd::new(1.8).unwrap(),
            )),
            Market::Total(TotalMarket::new(
                "total-1".to_string(),
                Line(2.5),
                Odd::new(1.9).unwrap(),
                Odd::new(1.9).unwrap(),
            )),
        ],
        Some("https://www.betano.pt/game".parse().unwrap()),
    )
}

#[test]
fn market_response_serializes_with_type_tag_and_odd_values() {
    let market = Market::Total(TotalMarket::new(
        "total-1".to_string(),
        Line(2.5),
        Odd::new(1.9).unwrap(),
        Odd::new(1.9).unwrap(),
    ));

    let value = serde_json::to_value(MarketResponse::from(&market)).unwrap();

    assert_eq!("Total", value["type"]);
    assert_eq!(2.5, value["line"]);
    assert_eq!(1.9, value["over"]["value"]);
    assert_eq!(1.9, value["under"]["value"]);
}

#[test]
fn market_response_serializes_every_variant_with_its_tag() {
    let variants = vec![
        (Market::match_result("mr", 2.0, 3.0, 4.0).unwrap(), "MatchResult"),
        (Market::moneyline("ml", 2.0, 1.8).unwrap(), "Moneyline"),
        (Market::double_chance("dc", 1.2, 1.5, 1.8).unwrap(), "DoubleChance"),
        (Market::handicap("hc", -1.0, 2.0, 3.0, 1.8).unwrap(), "Handicap"),
        (
            Market::asian_handicap("ah", -0.5, 2.0, 1.8).unwrap(),
            "AsianHandicap",
        ),
    ];

    for (market, tag) in variants {
        let value = serde_json::to_value(MarketResponse::from(&market)).unwrap();
        assert_eq!(tag, value["type"], "wrong tag for {tag}");
    }
}

#[test]
fn game_response_maps_game_metadata_markets_and_link() {
    let value: Value = serde_json::to_value(GameResponse::from(game(Platform::Betano))).unwrap();

    assert_eq!("g1", value["id"]);
    assert_eq!("Benfica", value["home_team"]);
    assert_eq!("Sporting", value["away_team"]);
    assert_eq!("Portugal", value["country"]);
    assert_eq!("Primeira Liga", value["competition"]);
    assert_eq!("Betano", value["platform"]);
    assert_eq!(2, value["markets"].as_array().unwrap().len());
    assert_eq!("https://www.betano.pt/game", value["link"]);
}

#[test]
fn game_response_has_null_link_when_missing() {
    let no_link = Game::new_with_id(
        "g1",
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        fixture_date(),
        Platform::Bwin,
        vec![],
        None,
    );

    let value: Value = serde_json::to_value(GameResponse::from(no_link)).unwrap();

    assert!(value["link"].is_null());
}

#[test]
fn cluster_response_exposes_games_representative_and_live_diffs() {
    let mut cluster = FixtureCluster::new(game_with_id("g1", Platform::Betano));
    cluster
        .try_to_add_game(game_with_id("g2", Platform::Polymarket))
        .expect("same fixture should cluster");

    let value: Value = serde_json::to_value(ClusterResponse::from(&Arc::new(cluster))).unwrap();

    assert_eq!(2, value["games"].as_array().unwrap().len());
    assert_eq!("Betano", value["representative_game"]["platform"]);
    assert!(value["id"].as_str().unwrap().contains("benfica"));
    assert!(value["updated_at"].as_str().is_some());
    // polymarket game has no implied probs derived from no -> live diffs empty
    assert!(value["live_diffs"].is_object());
}

#[test]
fn market_history_response_groups_points_by_market_variant_name() {
    use crate::domain::entities::MarketDataPoint;
    use std::collections::HashMap;

    let market = Market::Total(TotalMarket::new(
        "total-1".to_string(),
        Line(2.5),
        Odd::new(1.9).unwrap(),
        Odd::new(1.9).unwrap(),
    ));
    let point = Arc::new(MarketDataPoint::new_with_datetime(
        market,
        Utc.with_ymd_and_hms(2026, 5, 1, 12, 0, 0).unwrap(),
    ));

    let mut markets: HashMap<MarketType, Vec<Arc<crate::domain::entities::MarketDataPoint>>> =
        HashMap::new();
    markets
        .entry(MarketType::Total { line: 250 })
        .or_default()
        .push(point);

    let response = MarketHistoryResponse::from(("g1", &markets));
    let value: Value = serde_json::to_value(response).unwrap();

    assert_eq!("g1", value["game_id"]);
    assert_eq!(1, value["markets_by_type"]["Total"].as_array().unwrap().len());
    assert_eq!("Total", value["markets_by_type"]["Total"][0]["market"]["type"]);
}

#[test]
fn statistics_updated_response_keys_use_market_type_and_outcome_keys() {
    use crate::domain::services::cluster_statistics::{
        ClusterStatistics, StatisticsUpdated, StatisticsValues,
    };
    use std::collections::HashMap;

    let mut stats = ClusterStatistics::default();
    stats.add_diff(0.05);
    let mut inner = HashMap::new();
    inner.insert(Outcome::Home, StatisticsValues::from(&stats));
    let mut statistics = HashMap::new();
    statistics.insert(MarketType::Moneyline, inner);

    let update = StatisticsUpdated { statistics };
    let value: Value = serde_json::to_value(StatisticsUpdatedResponse::from(&update)).unwrap();

    let moneyline = &value["statistics"]["Moneyline"];
    assert!(moneyline["Home"].is_object());
    assert_eq!(1, moneyline["Home"]["samples"]);
    assert_eq!(0.05, moneyline["Home"]["mean_diff"]);
}

mod alert_response_tests {
    use serde_json::{Value, json};

    use crate::domain::{
        entities::{MarketType, Outcome},
        services::alert_service::{
            AlertConvergencyPayload, AlertEvent, MarketClusterDiffDivergencyPayload,
        },
        services::cluster_statistics::StatisticsValues,
    };

    use crate::infrastructure::server::dto::alert_response::AlertResponse;

    fn statistics() -> StatisticsValues {
        StatisticsValues {
            samples: 42,
            mean_diff: 0.05,
            median_diff: Some(0.05),
            p05_diff: Some(0.01),
            p25_diff: Some(0.03),
            p75_diff: Some(0.07),
            p95_diff: Some(0.09),
        }
    }

    #[test]
    fn divergency_event_serializes_with_type_and_statistics() {
        let event = AlertEvent::MarketClusterDiffDivergency(MarketClusterDiffDivergencyPayload {
            cluster_key: "benfica vs sporting".to_string(),
            cluster_mean_diff: 0.31,
            statistics: statistics(),
            market_type: MarketType::Moneyline,
            outcome: Outcome::Home,
        });

        let value: Value = serde_json::to_value(AlertResponse::from(&event)).unwrap();

        assert_eq!("MarketClusterDiffDivergency", value["type"]);
        assert_eq!("benfica vs sporting", value["payload"]["cluster_key"]);
        assert_eq!(0.31, value["payload"]["cluster_mean_diff"]);
        assert_eq!("Moneyline", value["payload"]["market_type"]);
        assert_eq!("Home", value["payload"]["outcome"]);
        assert_eq!(42, value["payload"]["statistics"]["samples"]);
        assert_eq!(0.09, value["payload"]["statistics"]["p95_diff"]);
    }

    #[test]
    fn convergency_event_serializes_with_probability_trace() {
        let event = AlertEvent::AlertConvergency(AlertConvergencyPayload {
            cluster_key: "benfica vs sporting".to_string(),
            cluster_mean_diff: 0.05,
            market_type: MarketType::Total { line: 250 },
            outcome: Outcome::Over,
            created_at: chrono::Utc::now(),
            initial_polymarket_impl_prob: 0.8,
            current_polymarket_impl_prob: 0.55,
        });

        let value: Value = serde_json::to_value(AlertResponse::from(&event)).unwrap();

        assert_eq!("AlertConvergency", value["type"]);
        assert_eq!(json!("Total:250"), value["payload"]["market_type"]);
        assert_eq!("Over", value["payload"]["outcome"]);
        assert_eq!(0.8, value["payload"]["initial_polymarket_impl_prob"]);
        assert_eq!(0.55, value["payload"]["current_polymarket_impl_prob"]);
    }
}

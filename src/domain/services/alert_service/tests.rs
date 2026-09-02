use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use num_traits::FromPrimitive;
use polymarket_client_sdk_v2::types::Decimal;
use uuid::Uuid;

use crate::domain::{
    ClusterService,
    entities::{
        Game, Market, MarketType, Odd, Outcome, Platform,
        markets::moneyline::MoneylineMarket,
    },
};
use crate::infrastructure::repositories::{
    connect_pool,
    fixture_cluster_repository::FixtureClusterRepository,
    game_repository::GameRepository,
};

use super::*;

async fn cluster_repository() -> Arc<FixtureClusterRepository> {
    let db_path = format!(
        "{}/alert_service_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    let pool = connect_pool(&db_path).await.unwrap();
    let game_repo = Arc::new(GameRepository::from_pool(pool.clone()));
    let repo = FixtureClusterRepository::from_pool(pool, game_repo);
    repo.run_migrations().await.unwrap();
    Arc::new(repo)
}

fn fixture_date() -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        NaiveTime::from_hms_milli_opt(20, 0, 0, 0).unwrap(),
    )
}

fn bookmaker_game(id: &str, home: f64, away: f64) -> Game {
    Game::new_with_id(
        id,
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        fixture_date(),
        Platform::Betano,
        vec![Market::Moneyline(MoneylineMarket::new(
            format!("{}-ml", id),
            Odd::new(home).unwrap(),
            Odd::new(away).unwrap(),
        ))],
        None,
    )
}

/// Polymarket odds are probabilities: `yes_prob` maps to the home outcome,
/// `no_prob` to the away outcome (the NO side of the home odd).
fn polymarket_game(id: &str, yes_prob: f64, no_prob: f64) -> Game {
    Game::new_with_id(
        id,
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        fixture_date(),
        Platform::Polymarket,
        vec![Market::Moneyline(MoneylineMarket::new(
            format!("{}-ml", id),
            Odd::new_from_prob(Decimal::from_f64(yes_prob).unwrap(), Decimal::from_f64(no_prob).unwrap()).unwrap(),
            Odd::new_from_prob(Decimal::from_f64(no_prob).unwrap(), Decimal::from_f64(yes_prob).unwrap()).unwrap(),
        ))],
        None,
    )
}

/// Twenty completed fixtures spread over 0.01..0.105 give a p05..p95 band of
/// roughly [0.015, 0.100] — wide enough to absorb float noise around 0.05.
fn seeded_statistics() -> Vec<f64> {
    (0..20).map(|i| 0.01 + i as f64 * 0.005).collect()
}

fn diffs_for_home(diff: f64) -> HashMap<MarketType, HashMap<Outcome, f64>> {
    let mut inner = HashMap::new();
    inner.insert(Outcome::Home, diff);
    let mut out = HashMap::new();
    out.insert(MarketType::Moneyline, inner);
    out
}

async fn setup() -> (Arc<ClusterService>, Arc<StatisticsService>, AlertService) {
    let repo = cluster_repository().await;
    let statistics_service = Arc::new(StatisticsService::new(repo.clone()));
    let cluster_service = Arc::new(
        ClusterService::new()
            .with_fixture_cluster_repository(repo)
            .with_statistics_service(statistics_service.clone()),
    );
    let alert_service = AlertService::new(cluster_service.clone(), statistics_service.clone());

    // give the spawned alert loop a chance to subscribe to cluster updates
    tokio::time::sleep(Duration::from_millis(100)).await;

    (cluster_service, statistics_service, alert_service)
}

#[tokio::test]
async fn emits_divergency_alert_when_polymarket_diff_leaves_statistical_band() {
    let (cluster_service, statistics_service, alert_service) = setup().await;
    for diff in seeded_statistics() {
        statistics_service.add_completed_fixture_diffs(diffs_for_home(diff));
    }

    let mut rx = alert_service.subscribe_to_new_alerts();

    // bookmaker implies 0.5 for home / 0.2 for away; polymarket implies 0.8 / 0.2
    // -> home diff 0.3 leaves the statistical band, away diff is exactly 0
    cluster_service.insert_games(vec![bookmaker_game("book-1", 2.0, 5.0)]);
    cluster_service.insert_games(vec![polymarket_game("poly-1", 0.8, 0.2)]);

    let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("timed out waiting for divergency alert")
        .unwrap();

    match &*event {
        AlertEvent::MarketClusterDiffDivergency(payload) => {
            assert_eq!(MarketType::Moneyline, payload.market_type);
            assert_eq!(Outcome::Home, payload.outcome);
            assert!((payload.cluster_mean_diff - 0.3).abs() < 1e-9);
            assert!(payload.statistics.p95_diff.is_some());
        }
        other => panic!("expected divergency alert, got {:?}", other),
    }
}

#[tokio::test]
async fn emits_convergency_alert_when_diff_returns_inside_band() {
    let (cluster_service, statistics_service, alert_service) = setup().await;
    for diff in seeded_statistics() {
        statistics_service.add_completed_fixture_diffs(diffs_for_home(diff));
    }

    let mut rx = alert_service.subscribe_to_new_alerts();

    cluster_service.insert_games(vec![bookmaker_game("book-1", 2.0, 5.0)]);
    cluster_service.insert_games(vec![polymarket_game("poly-1", 0.8, 0.2)]);

    // first event: divergency registers a monitor for the (market, outcome) pair
    let first = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("timed out waiting for divergency alert")
        .unwrap();
    assert!(matches!(
        &*first,
        AlertEvent::MarketClusterDiffDivergency(_)
    ));

    // polymarket converges towards the bookmaker: home diff 0.05 >= p05 and
    // derived-from-no diff 0.05 <= p95 bring the pair back inside the band
    cluster_service.insert_games(vec![polymarket_game("poly-1", 0.55, 0.45)]);

    let second = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("timed out waiting for convergency alert")
        .unwrap();

    match &*second {
        AlertEvent::AlertConvergency(payload) => {
            assert_eq!(MarketType::Moneyline, payload.market_type);
            assert_eq!(Outcome::Home, payload.outcome);
            assert!((payload.initial_polymarket_impl_prob - 0.8).abs() < 1e-9);
            assert!((payload.current_polymarket_impl_prob - 0.55).abs() < 1e-9);
        }
        other => panic!("expected convergency alert, got {:?}", other),
    }
}

#[tokio::test]
async fn no_alerts_when_diffs_stay_inside_the_statistical_band() {
    let (cluster_service, statistics_service, alert_service) = setup().await;
    for diff in seeded_statistics() {
        statistics_service.add_completed_fixture_diffs(diffs_for_home(diff));
    }

    let mut rx = alert_service.subscribe_to_new_alerts();

    // polymarket 0.55/0.45 vs bookmaker 0.5/0.4 keeps both outcomes inside the
    // band (diffs of 0.05 on the p05 edge, derived-from-no diffs of 0.05 <= p95)
    cluster_service.insert_games(vec![bookmaker_game("book-1", 2.0, 2.5)]);
    cluster_service.insert_games(vec![polymarket_game("poly-1", 0.55, 0.45)]);

    let result = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await;

    assert!(result.is_err(), "expected no alerts, got {:?}", result.err());
}

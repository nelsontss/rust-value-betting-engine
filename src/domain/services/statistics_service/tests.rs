use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use uuid::Uuid;

use crate::domain::entities::{Game, Market, MarketType, Odd, Outcome, Platform};
use crate::domain::entities::markets::moneyline::MoneylineMarket;
use crate::infrastructure::repositories::{
    connect_pool,
    fixture_cluster_repository::FixtureClusterRepository,
    game_repository::GameRepository,
};

use super::*;

async fn cluster_repository() -> Arc<FixtureClusterRepository> {
    let db_path = format!(
        "{}/statistics_service_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    let pool = connect_pool(&db_path).await.unwrap();
    let game_repo = Arc::new(GameRepository::from_pool(pool.clone()));
    let repo = FixtureClusterRepository::from_pool(pool, game_repo);
    repo.run_migrations().await.unwrap();
    Arc::new(repo)
}

fn moneyline_game(id: &str, platform: Platform, home: f64, away: f64) -> Game {
    Game::new_with_id(
        id,
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            NaiveTime::from_hms_milli_opt(18, 0, 0, 0).unwrap(),
        ),
        platform,
        vec![Market::Moneyline(MoneylineMarket::new(
            format!("{}-ml", id),
            Odd::new(home).unwrap(),
            Odd::new(away).unwrap(),
        ))],
        None,
    )
}

fn diffs_for_home(diff: f64) -> HashMap<MarketType, HashMap<Outcome, f64>> {
    let mut inner = HashMap::new();
    inner.insert(Outcome::Home, diff);
    let mut out = HashMap::new();
    out.insert(MarketType::Moneyline, inner);
    out
}

#[tokio::test]
async fn historical_statistics_start_empty() {
    let repo = cluster_repository().await;
    let service = StatisticsService::new(repo);

    let stats = service.get_historical_statistics();

    assert!(stats.is_empty());
}

#[tokio::test]
async fn add_completed_fixture_diffs_accumulates_and_broadcasts() {
    let repo = cluster_repository().await;
    let service = Arc::new(StatisticsService::new(repo));

    let mut rx = service.subscribe_to_statistics();

    service.add_completed_fixture_diffs(diffs_for_home(0.05));
    service.add_completed_fixture_diffs(diffs_for_home(0.07));

    let stats = service.get_historical_statistics();
    let moneyline = stats.get(&MarketType::Moneyline).unwrap();
    let home = moneyline.get(&Outcome::Home).unwrap();
    assert_eq!(2, home.samples);
    assert!((home.mean_diff - 0.06).abs() < 1e-9);
    // p05 interpolates 5% of the way between the two samples, p95 95% of the way
    assert!(matches!(home.p05_diff, Some(v) if (v - 0.051).abs() < 1e-9));
    assert!(matches!(home.p95_diff, Some(v) if (v - 0.069).abs() < 1e-9));

    // both updates broadcast a statistics snapshot
    let first = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        1,
        first.statistics[&MarketType::Moneyline][&Outcome::Home].samples
    );
    let second = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        2,
        second.statistics[&MarketType::Moneyline][&Outcome::Home].samples
    );
}

#[tokio::test]
async fn load_historical_diffs_rebuilds_distribution_from_repository() {
    let repo = cluster_repository().await;
    let cluster = crate::domain::entities::FixtureCluster::new(moneyline_game(
        "g1",
        Platform::Betano,
        2.0,
        1.8,
    ));
    repo.insert_cluster(&cluster).await.unwrap();
    repo.insert_cluster_diffs(&cluster.key(), &diffs_for_home(0.05))
        .await
        .unwrap();

    let service = StatisticsService::new(repo);
    service.load_historical_diffs().await.unwrap();

    let stats = service.get_historical_statistics();
    let home = &stats[&MarketType::Moneyline][&Outcome::Home];
    assert_eq!(1, home.samples);
    assert!((home.mean_diff - 0.05).abs() < 1e-9);
}

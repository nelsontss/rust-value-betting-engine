use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use uuid::Uuid;

use crate::{
    domain::entities::{Game, Market, Odd, Platform, markets::moneyline::MoneylineMarket},
    infrastructure::repositories::connect_pool,
};

use super::*;

async fn repository() -> FixtureClusterRepository {
    let db_path = format!(
        "{}/fixture_cluster_repository_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    repository_at(&db_path).await
}

async fn repository_at(db_path: &str) -> FixtureClusterRepository {
    let pool = connect_pool(db_path).await.unwrap();
    let repo = FixtureClusterRepository::from_pool(
        pool.clone(),
        Arc::new(GameRepository::from_pool(pool)),
    );
    repo.run_migrations().await.unwrap();
    repo
}

fn game(id: &str, platform: Platform, home: f64, away: f64) -> Game {
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
    )
}

fn future_game(id: &str, platform: Platform, home: f64, away: f64) -> Game {
    Game::new_with_id(
        id,
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            NaiveTime::from_hms_milli_opt(18, 0, 0, 0).unwrap(),
        ),
        platform,
        vec![Market::Moneyline(MoneylineMarket::new(
            format!("{}-ml", id),
            Odd::new(home).unwrap(),
            Odd::new(away).unwrap(),
        ))],
    )
}

async fn diff_row_count(repo: &FixtureClusterRepository) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM fixture_cluster_diff")
        .fetch_one(&repo.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn insert_cluster_diffs_persists_and_round_trips_into_loaded_cluster() {
    let repo = repository().await;
    let mut cluster = FixtureCluster::new(game("g1", Platform::Betano, 2.0, 1.8));
    cluster
        .try_to_add_game(game("g2", Platform::Polymarket, 2.1, 1.7))
        .unwrap();

    repo.insert_cluster(&cluster).await.unwrap();

    let mut diffs = HashMap::new();
    diffs.insert((MarketType::Moneyline, Outcome::Home), 0.05);
    diffs.insert((MarketType::Moneyline, Outcome::Away), -0.03);
    repo.insert_cluster_diffs(&cluster.key(), &diffs)
        .await
        .unwrap();

    let loaded = repo.get_cluster(&cluster.key()).await.unwrap().unwrap();
    assert_eq!(diffs, loaded.statistics_diffs());
    assert_eq!(2, repo.get_all_cluster_diffs().await.unwrap().len());
}

#[tokio::test]
async fn insert_cluster_diffs_upserts_same_fixture_market_outcome() {
    let repo = repository().await;
    let cluster = FixtureCluster::new(game("g1", Platform::Betano, 2.0, 1.8));
    repo.insert_cluster(&cluster).await.unwrap();

    let mut first = HashMap::new();
    first.insert((MarketType::Moneyline, Outcome::Home), 0.10);
    repo.insert_cluster_diffs(&cluster.key(), &first)
        .await
        .unwrap();

    let mut second = HashMap::new();
    second.insert((MarketType::Moneyline, Outcome::Home), 0.02);
    second.insert((MarketType::Moneyline, Outcome::Away), -0.01);
    repo.insert_cluster_diffs(&cluster.key(), &second)
        .await
        .unwrap();

    assert_eq!(2, diff_row_count(&repo).await);

    let loaded = repo.get_cluster(&cluster.key()).await.unwrap().unwrap();
    assert_eq!(second, loaded.statistics_diffs());
}

#[tokio::test]
async fn get_all_cluster_diffs_aggregates_across_fixtures() {
    let repo = repository().await;
    let c1 = FixtureCluster::new(game("g1", Platform::Betano, 2.0, 1.8));
    let c2 = FixtureCluster::new(future_game("g2", Platform::Betano, 2.0, 1.8));
    repo.insert_cluster(&c1).await.unwrap();
    repo.insert_cluster(&c2).await.unwrap();

    let mut d1 = HashMap::new();
    d1.insert((MarketType::Moneyline, Outcome::Home), 0.05);
    repo.insert_cluster_diffs(&c1.key(), &d1).await.unwrap();

    let mut d2 = HashMap::new();
    d2.insert((MarketType::Moneyline, Outcome::Home), -0.02);
    repo.insert_cluster_diffs(&c2.key(), &d2).await.unwrap();

    let all = repo.get_all_cluster_diffs().await.unwrap();

    assert_eq!(2, all.len());
    assert_eq!(0.05, all[0].2);
    assert_eq!(-0.02, all[1].2);
}

#[tokio::test]
async fn insert_cluster_round_trips_closed_flag() {
    let repo = repository().await;
    let mut cluster = FixtureCluster::new(game("g1", Platform::Betano, 2.0, 1.8));
    assert!(!cluster.is_closed());

    repo.insert_cluster(&cluster).await.unwrap();
    let loaded = repo.get_cluster(&cluster.key()).await.unwrap().unwrap();
    assert!(!loaded.is_closed());

    cluster.close();
    repo.insert_cluster(&cluster).await.unwrap();

    let loaded = repo.get_cluster(&cluster.key()).await.unwrap().unwrap();
    assert!(loaded.is_closed());
}

#[tokio::test]
async fn migration_adds_closed_column_to_legacy_fixture_cluster_table() {
    let db_path = format!(
        "{}/fixture_cluster_legacy_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    let pool = connect_pool(&db_path).await.unwrap();

    sqlx::query(
        "CREATE TABLE games (
            id TEXT PRIMARY KEY,
            home_team TEXT NOT NULL,
            away_team TEXT NOT NULL,
            country TEXT NOT NULL,
            competition TEXT NOT NULL,
            platform TEXT NOT NULL,
            date INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE fixture_cluster (
            key TEXT PRIMARY KEY,
            representative_game_id TEXT REFERENCES games(id) ON DELETE SET NULL,
            game_date INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    drop(pool);

    let repo = repository_at(&db_path).await;
    let cluster = FixtureCluster::new(game("g1", Platform::Betano, 2.0, 1.8));

    repo.insert_cluster(&cluster).await.unwrap();

    let loaded = repo.get_cluster(&cluster.key()).await.unwrap().unwrap();
    assert!(!loaded.is_closed());
}

#[tokio::test]
async fn insert_then_get_round_trips_cluster_with_games() {
    let repo = repository().await;
    let cluster = FixtureCluster::new(game("g1", Platform::Betano, 2.0, 1.8));

    repo.insert_cluster(&cluster).await.unwrap();

    let loaded = repo.get_cluster(&cluster.key()).await.unwrap().unwrap();
    assert_eq!(loaded.key(), cluster.key());
    assert_eq!(loaded.game_count(), 1);

    let loaded_game = loaded.get_game("g1").unwrap();
    assert_eq!(loaded_game.platform(), Platform::Betano);
    assert_eq!(loaded_game.home_team(), "Benfica");
    assert_eq!(loaded_game.away_team(), "Sporting");
}

#[tokio::test]
async fn insert_cluster_with_multiple_games_round_trips() {
    let repo = repository().await;
    let mut cluster = FixtureCluster::new(game("g1", Platform::Betano, 2.0, 1.8));
    cluster
        .try_to_add_game(game("g2", Platform::Polymarket, 2.1, 1.7))
        .unwrap();

    repo.insert_cluster(&cluster).await.unwrap();

    let loaded = repo.get_cluster(&cluster.key()).await.unwrap().unwrap();
    assert_eq!(loaded.game_count(), 2);
    assert!(loaded.get_game("g1").is_some());
    assert!(loaded.get_game("g2").is_some());
}

#[tokio::test]
async fn get_cluster_returns_none_for_unknown_key() {
    let repo = repository().await;

    assert!(repo.get_cluster("does-not-exist").await.unwrap().is_none());
}

#[tokio::test]
async fn get_all_clusters_returns_inserted_clusters() {
    let repo = repository().await;
    let cluster = FixtureCluster::new(game("g1", Platform::Betano, 2.0, 1.8));

    repo.insert_cluster(&cluster).await.unwrap();

    let all = repo.get_all_clusters().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].key(), cluster.key());
}

#[tokio::test]
async fn get_future_clusters_excludes_past_clusters() {
    let repo = repository().await;
    let past = FixtureCluster::new(game("g1", Platform::Betano, 2.0, 1.8));
    repo.insert_cluster(&past).await.unwrap();

    let future = FixtureCluster::new(future_game("g2", Platform::Betano, 2.0, 1.8));
    repo.insert_cluster(&future).await.unwrap();

    let clusters = repo.get_future_clusters().await.unwrap();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].key(), future.key());
}

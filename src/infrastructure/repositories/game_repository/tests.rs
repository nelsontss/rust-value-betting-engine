use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use uuid::Uuid;

use crate::domain::entities::{
    Odd,
    markets::{Line, moneyline::MoneylineMarket, total::TotalMarket},
};

use super::*;

async fn repository() -> GameRepository {
    let db_path = format!(
        "{}/game_repository_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    let pool = SqlitePool::connect(&format!("{}?mode=rwc", db_path))
        .await
        .unwrap();
    let repo = GameRepository { pool };
    repo.run_migrations().await.unwrap();
    repo
}

fn fixture_date() -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        NaiveTime::from_hms_milli_opt(18, 0, 0, 0).unwrap(),
    )
}

fn build_game(platform: Platform) -> Game {
    Game::new(
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
        None,
    )
}

#[tokio::test]
async fn insert_then_get_round_trips_game_and_markets() {
    let repo = repository().await;
    let game = build_game(Platform::Betano);

    repo.insert_game(&game).await.unwrap();

    let loaded = repo.get_game(&game.id).await.unwrap().unwrap();

    assert_eq!(game.id, loaded.id);
    assert_eq!(game.date, loaded.date);
    assert_eq!(game.home_team(), loaded.home_team());
    assert_eq!(game.away_team(), loaded.away_team());
    assert_eq!(game.country(), loaded.country());
    assert_eq!(game.competition(), loaded.competition());
    assert_eq!(game.platform(), loaded.platform());
    assert_eq!(game.markets(), loaded.markets());
}

#[tokio::test]
async fn platform_round_trips_for_all_variants() {
    for platform in [
        Platform::Betano,
        Platform::LeBull,
        Platform::Bwin,
        Platform::Polymarket,
    ] {
        let repo = repository().await;
        let game = build_game(platform);

        repo.insert_game(&game).await.unwrap();
        let loaded = repo.get_game(&game.id).await.unwrap().unwrap();

        assert_eq!(platform, loaded.platform());
    }
}

#[tokio::test]
async fn get_game_returns_none_for_unknown_id() {
    let repo = repository().await;

    let loaded = repo.get_game("does-not-exist").await.unwrap();

    assert!(loaded.is_none());
}

#[tokio::test]
async fn update_game_replaces_markets() {
    let repo = repository().await;
    let mut game = build_game(Platform::Betano);
    repo.insert_game(&game).await.unwrap();

    game.update_markets(vec![Market::Moneyline(MoneylineMarket::new(
        "ml-1".to_string(),
        Odd::new(2.4).unwrap(),
        Odd::new(1.6).unwrap(),
    ))]);

    repo.update_game(&game).await.unwrap();

    let loaded = repo.get_game(&game.id).await.unwrap().unwrap();
    assert_eq!(game.markets(), loaded.markets());
}

#[tokio::test]
async fn get_game_markets_history_appends_each_tick_in_order() {
    let repo = repository().await;
    let mut game = build_game(Platform::Betano);
    repo.insert_game(&game).await.unwrap();

    let updated_market = Market::Moneyline(MoneylineMarket::new(
        "ml-1".to_string(),
        Odd::new(2.4).unwrap(),
        Odd::new(1.6).unwrap(),
    ));
    game.update_markets(vec![updated_market.clone()]);
    repo.update_game(&game).await.unwrap();

    let history = repo.get_game_markets_history(&game.id).await.unwrap();

    assert_eq!(3, history.len());
    assert!(
        history
            .iter()
            .any(|point| point.market() == &updated_market)
    );
    assert_eq!(
        2,
        history
            .iter()
            .filter(|point| matches!(point.market(), Market::Moneyline(_)))
            .count()
    );
    assert_eq!(
        1,
        history
            .iter()
            .filter(|point| matches!(point.market(), Market::Total(_)))
            .count()
    );

    for pair in history.windows(2) {
        assert!(pair[0].datetime() <= pair[1].datetime());
    }
}

#[tokio::test]
async fn upsert_game_inserts_and_updates_atomically() {
    let repo = repository().await;
    let game = build_game(Platform::Betano);

    repo.upsert_game(&game).await.unwrap();
    let loaded = repo.get_game(&game.id).await.unwrap().unwrap();
    assert_eq!(2, loaded.markets().len());

    // upsert again with new markets must not fail with a unique constraint
    let updated = Game::new_with_id(
        &game.id,
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        fixture_date(),
        Platform::Betano,
        vec![Market::Moneyline(MoneylineMarket::new(
            "ml-1".to_string(),
            Odd::new(3.0).unwrap(),
            Odd::new(1.4).unwrap(),
        ))],
        None,
    );
    repo.upsert_game(&updated).await.unwrap();

    let loaded = repo.get_game(&game.id).await.unwrap().unwrap();
    assert_eq!(1, loaded.markets().len());
}

#[tokio::test]
async fn upsert_game_preserves_existing_link_when_new_link_is_none() {
    let repo = repository().await;
    let with_link = Game::new_with_id(
        "link-game",
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        fixture_date(),
        Platform::Betano,
        vec![],
        Some("https://www.betano.pt/game".parse().unwrap()),
    );
    repo.upsert_game(&with_link).await.unwrap();

    let without_link = Game::new_with_id(
        "link-game",
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        fixture_date(),
        Platform::Betano,
        vec![],
        None,
    );
    repo.upsert_game(&without_link).await.unwrap();

    let loaded = repo.get_game("link-game").await.unwrap().unwrap();
    assert!(loaded.link().is_some());
}

#[tokio::test]
async fn insert_game_with_duplicate_id_fails() {
    let repo = repository().await;
    let game = build_game(Platform::Betano);

    repo.insert_game(&game).await.unwrap();
    assert!(repo.insert_game(&game).await.is_err());
}

#[tokio::test]
async fn all_market_variants_round_trip_through_the_database() {
    let repo = repository().await;
    let game = Game::new(
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        fixture_date(),
        Platform::Betano,
        vec![
            Market::match_result("mr-1", 2.0, 3.0, 4.0).unwrap(),
            Market::double_chance("dc-1", 1.2, 1.5, 1.8).unwrap(),
            Market::handicap("hc-1", -1.0, 2.0, 3.0, 1.8).unwrap(),
            Market::asian_handicap("ah-1", -0.5, 2.0, 1.8).unwrap(),
        ],
        None,
    );

    repo.insert_game(&game).await.unwrap();

    let loaded = repo.get_game(&game.id).await.unwrap().unwrap();
    assert_eq!(4, loaded.markets().len());
    assert_eq!(game.markets(), loaded.markets());
}

#[tokio::test]
async fn market_rows_with_missing_odds_fail_to_reconstruct() {
    let repo = repository().await;
    let game = build_game(Platform::Betano);
    repo.insert_game(&game).await.unwrap();

    // corrupt the moneyline row by nulling the away odd
    sqlx::query("UPDATE markets SET away = NULL WHERE market_id = 'ml-1'")
        .execute(&repo.pool)
        .await
        .unwrap();

    assert!(repo.get_game(&game.id).await.is_err());
}

#[tokio::test]
async fn market_rows_with_unknown_market_type_fail_to_reconstruct() {
    let repo = repository().await;
    let game = build_game(Platform::Betano);
    repo.insert_game(&game).await.unwrap();

    sqlx::query("UPDATE markets SET market_type = 'Exotic'")
        .execute(&repo.pool)
        .await
        .unwrap();

    assert!(repo.get_game(&game.id).await.is_err());
}

#[tokio::test]
async fn games_with_unknown_platform_fail_to_load() {
    let repo = repository().await;
    let game = build_game(Platform::Betano);
    repo.insert_game(&game).await.unwrap();

    sqlx::query("UPDATE games SET platform = 'mystic'")
        .execute(&repo.pool)
        .await
        .unwrap();

    assert!(repo.get_game(&game.id).await.is_err());
}

#[tokio::test]
async fn history_rows_with_invalid_timestamps_fail_to_load() {
    let repo = repository().await;
    let game = build_game(Platform::Betano);
    repo.insert_game(&game).await.unwrap();

    // corrupt the created_at timestamp beyond what chrono can represent
    sqlx::query("UPDATE markets SET created_at = 99999999999999")
        .execute(&repo.pool)
        .await
        .unwrap();

    assert!(repo.get_game_markets_history(&game.id).await.is_err());
}

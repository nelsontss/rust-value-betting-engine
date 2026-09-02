use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use uuid::Uuid;

use crate::domain::entities::{
    Market, Odd, Platform,
    markets::{Line, moneyline::MoneylineMarket, total::TotalMarket},
};
use crate::infrastructure::repositories::{connect_pool, game_repository::GameRepository};

use super::*;

async fn repository() -> Arc<GameRepository> {
    let db_path = format!(
        "{}/market_service_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    let pool = connect_pool(&db_path).await.unwrap();
    let repo = GameRepository::from_pool(pool);
    repo.run_migrations().await.unwrap();
    Arc::new(repo)
}

fn fixture_date() -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        NaiveTime::from_hms_milli_opt(18, 0, 0, 0).unwrap(),
    )
}

fn game_with_markets(id: &str, markets: Vec<Market>) -> crate::domain::entities::Game {
    crate::domain::entities::Game::new_with_id(
        id,
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        fixture_date(),
        Platform::Betano,
        markets,
        None,
    )
}

#[tokio::test]
async fn get_game_markets_history_groups_data_points_by_market_type() {
    let repo = repository().await;
    let service = MarketService::new(repo.clone());

    let game = game_with_markets(
        "g1",
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
    );
    repo.insert_game(&game).await.unwrap();

    let updated = game_with_markets(
        "g1",
        vec![Market::Moneyline(MoneylineMarket::new(
            "ml-1".to_string(),
            Odd::new(2.2).unwrap(),
            Odd::new(1.7).unwrap(),
        ))],
    );
    repo.update_game(&updated).await.unwrap();

    let history = service.get_game_markets_history("g1").await.unwrap().unwrap();

    assert_eq!(2, history.len());
    assert_eq!(2, history[&MarketType::Moneyline].len());
    assert_eq!(1, history[&MarketType::Total { line: 250 }].len());
}

#[tokio::test]
async fn get_game_markets_history_returns_none_for_unknown_game() {
    let repo = repository().await;
    let service = MarketService::new(repo);

    let history = service
        .get_game_markets_history("does-not-exist")
        .await
        .unwrap();

    assert!(history.is_none());
}

#[tokio::test]
async fn send_new_market_update_broadcasts_one_point_per_market() {
    let repo = repository().await;
    let service = Arc::new(MarketService::new(repo));

    let mut rx = service.subscribe_to_game_market_updates();

    let mut markets = std::collections::HashMap::new();
    markets.insert(
        MarketType::Moneyline,
        Market::Moneyline(MoneylineMarket::new(
            "ml-1".to_string(),
            Odd::new(2.0).unwrap(),
            Odd::new(1.8).unwrap(),
        )),
    );
    markets.insert(
        MarketType::DoubleChance,
        Market::double_chance("dc-1", 1.2, 1.5, 1.8).unwrap(),
    );

    service.send_new_market_update("g1", &markets);

    let first = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    let second = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!("g1", first.0);
    assert_eq!("g1", second.0);
    let types = [
        MarketType::from(first.1.market()),
        MarketType::from(second.1.market()),
    ];
    assert!(types.contains(&MarketType::Moneyline));
    assert!(types.contains(&MarketType::DoubleChance));
}

use polymarket_client_sdk_v2::clob::types::Side;
use polymarket_client_sdk_v2::types::Decimal;
use uuid::Uuid;

use crate::domain::entities::trade::{Trade, TradeStatus, TradeStrategy};
use crate::infrastructure::repositories::{connect_pool, trade_repository::TradeRepository};

async fn repository() -> TradeRepository {
    let db_path = format!(
        "{}/trade_repository_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    let repo = TradeRepository::from_pool(connect_pool(&db_path).await.unwrap());
    repo.run_migrations().await.unwrap();
    repo
}

fn paper_trade(id: &str) -> Trade {
    Trade {
        id: id.to_string(),
        market_id: "market-1".to_string(),
        token_id: "12345".to_string(),
        side: Side::Buy,
        size: Decimal::new(10, 0),
        entry_price: Decimal::new(45, 2),
        entry_time: 1_000,
        exit_price: None,
        exit_time: None,
        pnl: None,
        status: TradeStatus::Open,
        paper: true,
        created_at: 1_000,
        updated_at: 1_000,
        buy_order_id: Some("buy-order-1".to_string()),
        sell_order_id: None,
        strategy: TradeStrategy::DrawTimeDecay,
    }
}

#[tokio::test]
async fn insert_then_get_round_trips_an_open_trade() {
    let repo = repository().await;
    let trade = paper_trade("t-1");

    repo.insert_trade(&trade).await.unwrap();
    let loaded = repo.get_trade("t-1").await.unwrap().unwrap();

    assert_eq!("t-1", loaded.id);
    assert_eq!("market-1", loaded.market_id);
    assert_eq!("12345", loaded.token_id);
    assert_eq!("buy", loaded.side_str());
    assert_eq!(Decimal::new(10, 0), loaded.size);
    assert_eq!(Decimal::new(45, 2), loaded.entry_price);
    assert_eq!(1_000, loaded.entry_time);
    assert_eq!(TradeStatus::Open, loaded.status);
    assert!(loaded.paper);
    assert_eq!(TradeStrategy::DrawTimeDecay, loaded.strategy);
    assert_eq!(Some("buy-order-1".to_string()), loaded.buy_order_id);
    assert_eq!(None, loaded.exit_price);
    assert_eq!(None, loaded.pnl);
}

#[tokio::test]
async fn get_trade_returns_none_for_unknown_id() {
    let repo = repository().await;

    assert!(repo.get_trade("missing").await.unwrap().is_none());
}

#[tokio::test]
async fn update_trade_persists_the_closed_state() {
    let repo = repository().await;
    let mut trade = paper_trade("t-1");
    repo.insert_trade(&trade).await.unwrap();

    trade.close_trade(Decimal::new(60, 2), 2_000, Some("sell-order-1".to_string()));
    repo.update_trade(&trade).await.unwrap();

    let loaded = repo.get_trade("t-1").await.unwrap().unwrap();
    assert_eq!(TradeStatus::Closed, loaded.status);
    assert_eq!(Some(Decimal::new(60, 2)), loaded.exit_price);
    assert_eq!(Some(2_000), loaded.exit_time);
    assert_eq!(Some(Decimal::new(150, 2)), loaded.pnl);
    assert_eq!(Some("sell-order-1".to_string()), loaded.sell_order_id);
}

#[tokio::test]
async fn get_open_trades_returns_only_open_trades() {
    let repo = repository().await;
    let mut closed = paper_trade("t-closed");
    closed.close_trade(Decimal::new(60, 2), 2_000, None);

    repo.insert_trade(&paper_trade("t-open-1")).await.unwrap();
    repo.insert_trade(&closed).await.unwrap();
    repo.insert_trade(&paper_trade("t-open-2")).await.unwrap();

    let open = repo.get_open_trades().await.unwrap();

    let mut ids: Vec<String> = open.iter().map(|t| t.id.clone()).collect();
    ids.sort();
    assert_eq!(vec!["t-open-1".to_string(), "t-open-2".to_string()], ids);
}

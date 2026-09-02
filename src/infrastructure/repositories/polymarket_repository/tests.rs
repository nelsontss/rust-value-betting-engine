use uuid::Uuid;

use crate::infrastructure::repositories::connect_pool;

use super::*;

async fn repository() -> PolymarketRepository {
    let db_path = format!(
        "{}/polymarket_repository_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    let repo = PolymarketRepository::from_pool(connect_pool(&db_path).await.unwrap());
    repo.run_migrations().await.unwrap();
    repo
}

#[tokio::test]
async fn events_round_trip() {
    let repo = repository().await;

    repo.insert_event(
        "evt-1",
        "Benfica vs Porto",
        Some("benfica-vs-porto"),
        None,
        None,
        Some("Benfica"),
        Some("Porto"),
        Some("[\"soccer\"]"),
        Some(1000.0),
        None,
        false,
        Some("2026-05-10T18:00:00Z"),
        Some("2026-05-10T18:00:00Z"),
        Some("2026-05-10T20:00:00Z"),
        "2026-05-01T00:00:00Z",
    )
    .await
    .unwrap();

    assert_eq!(repo.event_count().await.unwrap(), 1);

    let events = repo
        .get_events_in_date_range("2026-05-01", "2026-05-31", "ASC")
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, "evt-1");
    assert_eq!(events[0].home_team.as_deref(), Some("Benfica"));
    assert_eq!(
        events[0].start_time.as_deref(),
        Some("2026-05-10T18:00:00Z")
    );

    let desc = repo
        .get_events_in_date_range("2026-05-01", "2026-05-31", "DESC")
        .await
        .unwrap();
    assert_eq!(desc.len(), 1);
}

#[tokio::test]
async fn markets_round_trip() {
    let repo = repository().await;

    repo.insert_event(
        "evt-1",
        "Benfica vs Porto",
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        true,
        None,
        Some("2026-05-10T18:00:00Z"),
        None,
        "2026-05-01T00:00:00Z",
    )
    .await
    .unwrap();

    repo.insert_market(
        "market-1",
        "evt-1",
        "Will Benfica win?",
        Some("winner-draw-benfica-braga"),
        None,
        Some("closed"),
        Some("winner"),
        Some(500.0),
        None,
        Some(100.0),
        None,
        Some("yes-token-1"),
        Some("no-token-1"),
        Some(0.55),
        Some(0.45),
    )
    .await
    .unwrap();

    let without_prices = repo.get_markets_without_prices().await.unwrap();
    assert_eq!(without_prices.len(), 1);
    assert_eq!(without_prices[0].id, "market-1");
    assert_eq!(
        without_prices[0].match_start.as_deref(),
        Some("2026-05-10T18:00:00Z")
    );

    let for_event = repo.get_markets_for_event("evt-1").await.unwrap();
    assert_eq!(for_event.len(), 1);

    repo.set_market_has_prices("market-1").await.unwrap();
    assert_eq!(repo.get_markets_without_prices().await.unwrap().len(), 0);

    let draw = repo.get_draw_markets().await.unwrap();
    assert_eq!(draw.len(), 1);
}

#[tokio::test]
async fn candles_round_trip() {
    let repo = repository().await;

    repo.insert_event(
        "evt-1",
        "Benfica vs Porto",
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        true,
        None,
        Some("2026-05-10T18:00:00Z"),
        None,
        "2026-05-01T00:00:00Z",
    )
    .await
    .unwrap();

    repo.insert_market(
        "market-1",
        "evt-1",
        "Will Benfica win?",
        Some("winner-draw-benfica-braga"),
        None,
        None,
        Some("winner"),
        None,
        None,
        None,
        None,
        Some("yes-token-1"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    repo.insert_price_candle("market-1", "yes-token-1", 123, 0.5, 0.6, 0.45, 0.55, 100.0)
        .await
        .unwrap();
    repo.insert_price_candle("market-1", "yes-token-1", 456, 0.55, 0.6, 0.5, 0.6, 200.0)
        .await
        .unwrap();

    assert_eq!(repo.price_count("market-1").await.unwrap(), 2);

    let candles = repo.get_candles_for_market("market-1").await.unwrap();
    assert_eq!(candles.len(), 2);
    assert_eq!(candles.first().unwrap().timestamp, 123);
    assert_eq!(candles.first().unwrap().open, 0.5);
    assert_eq!(candles.last().unwrap().close, 0.6);
}

#[tokio::test]
async fn insert_event_replaces_existing_by_id() {
    let repo = repository().await;

    repo.insert_event(
        "evt-1",
        "Benfica vs Porto",
        Some("old"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        true,
        Some("2026-05-05"),
        None,
        None,
        "2026-05-01T00:00:00Z",
    )
    .await
    .unwrap();
    repo.insert_event(
        "evt-1",
        "Benfica vs Porto",
        Some("new-slug"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        true,
        Some("2026-05-05"),
        None,
        None,
        "2026-05-02T00:00:00Z",
    )
    .await
    .unwrap();

    assert_eq!(repo.event_count().await.unwrap(), 1);

    let events = repo
        .get_events_in_date_range("2026-05-01", "2026-05-31", "ASC")
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, "evt-1");
    assert_eq!(events[0].title.as_deref(), Some("Benfica vs Porto"));
}

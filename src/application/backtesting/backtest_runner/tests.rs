use uuid::Uuid;

use crate::infrastructure::repositories::{PolymarketRepository, connect_pool};

use super::*;

async fn repository() -> PolymarketRepository {
    let db_path = format!(
        "{}/backtest_runner_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    let repo = PolymarketRepository::from_pool(connect_pool(&db_path).await.unwrap());
    repo.run_migrations().await.unwrap();
    repo
}

const KICKOFF: &str = "2026-05-10T18:00:00Z";

async fn seed_draw_market_with_candles(
    repo: &PolymarketRepository,
    market_id: &str,
    with_match_start: bool,
    closes: &[(i64, f64)],
) {
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
        Some(KICKOFF),
        if with_match_start {
            Some(KICKOFF)
        } else {
            None
        },
        None,
        "2026-05-01T00:00:00Z",
    )
    .await
    .unwrap();

    repo.insert_market(
        market_id,
        "evt-1",
        "Will the match end in a draw?",
        Some("draw-benfica-porto"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("yes-token"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    for (timestamp, close) in closes {
        repo.insert_price_candle(market_id, "yes-token", *timestamp, *close, *close, *close, *close, 1.0)
            .await
            .unwrap();
    }
}

fn config() -> BacktestConfig {
    BacktestConfig {
        resolution_minutes: 10,
        from: None,
        to: None,
    }
}

fn kickoff_ms() -> i64 {
    chrono::DateTime::parse_from_rfc3339(KICKOFF).unwrap().timestamp_millis()
}

#[tokio::test]
async fn run_reports_zero_metrics_on_empty_database() {
    let repo = repository().await;

    let metrics = BacktestRunner::run(&repo, &config()).await.unwrap();

    assert_eq!(0, metrics.total_markets);
    assert_eq!(0, metrics.total_trades);
    assert_eq!(0.0, metrics.total_pnl);
    assert_eq!(0.0, metrics.win_rate);
}

#[tokio::test]
async fn run_simulates_draw_value_trades_from_stored_candles() {
    let repo = repository().await;
    let start = kickoff_ms();
    // entry candle 5 minutes before kickoff (close 0.10 -> buy at 0.09),
    // exit candle 12 minutes after kickoff (close 0.05)
    seed_draw_market_with_candles(
        &repo,
        "market-1",
        true,
        &[
            (start - 5 * 60_000, 0.10),
            (start + 12 * 60_000, 0.05),
        ],
    )
    .await;

    let metrics = BacktestRunner::run(&repo, &config()).await.unwrap();

    assert_eq!(1, metrics.total_markets);
    assert_eq!(1, metrics.markets_with_data);
    assert_eq!(1, metrics.total_trades);
    assert_eq!(0, metrics.winning_trades);
    assert_eq!(1, metrics.losing_trades);
    assert_eq!(0.0, metrics.win_rate);
    assert!((metrics.total_pnl - (-0.04)).abs() < 1e-9);
    assert!((metrics.avg_pnl - (-0.04)).abs() < 1e-9);
}

#[tokio::test]
async fn run_counts_winning_trade_when_price_decays_towards_resolution() {
    let repo = repository().await;
    let start = kickoff_ms();
    seed_draw_market_with_candles(
        &repo,
        "market-1",
        true,
        &[
            (start - 5 * 60_000, 0.10),
            (start + 12 * 60_000, 0.15),
        ],
    )
    .await;

    let metrics = BacktestRunner::run(&repo, &config()).await.unwrap();

    assert_eq!(1, metrics.total_trades);
    assert_eq!(1, metrics.winning_trades);
    assert!((metrics.total_pnl - 0.06).abs() < 1e-9);
}

#[tokio::test]
async fn run_skips_markets_without_match_start_but_still_counts_them() {
    let repo = repository().await;
    let start = kickoff_ms();
    seed_draw_market_with_candles(
        &repo,
        "market-1",
        false,
        &[(start - 5 * 60_000, 0.10)],
    )
    .await;

    let metrics = BacktestRunner::run(&repo, &config()).await.unwrap();

    assert_eq!(1, metrics.total_markets);
    assert_eq!(1, metrics.markets_with_data);
    assert_eq!(0, metrics.total_trades);
}

#[tokio::test]
async fn run_ignores_markets_that_are_not_draw_markets() {
    let repo = repository().await;
    let start = kickoff_ms();

    // market whose slug has no "draw" -> derived_type is NULL -> excluded
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
        Some(KICKOFF),
        Some(KICKOFF),
        None,
        "2026-05-01T00:00:00Z",
    )
    .await
    .unwrap();
    repo.insert_market(
        "market-1",
        "evt-1",
        "Will Benfica win?",
        Some("winner-benfica-porto"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("yes-token"),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    repo.insert_price_candle("market-1", "yes-token", start, 0.10, 0.10, 0.10, 0.10, 1.0)
        .await
        .unwrap();

    let metrics = BacktestRunner::run(&repo, &config()).await.unwrap();

    assert_eq!(0, metrics.total_markets);
}

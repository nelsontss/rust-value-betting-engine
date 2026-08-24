pub mod fixture_cluster_repository;
pub mod game_repository;
pub mod polymarket_repository;
pub mod trade_repository;

use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

use crate::shared::error::Result;

pub use polymarket_repository::EventRow;
pub use polymarket_repository::MarketRow;
pub use polymarket_repository::PolymarketRepository;

pub async fn connect_pool(db_path: &str) -> Result<sqlx::SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(10));

    Ok(SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?)
}

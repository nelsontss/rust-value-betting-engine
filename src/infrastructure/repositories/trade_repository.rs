use crate::{domain::entities::trade::Trade, shared::error::Result};
use sqlx::SqlitePool;

#[cfg(test)]
mod tests;

pub struct TradeRepository {
    pool: SqlitePool,
}

impl TradeRepository {
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS trades (
                id TEXT PRIMARY KEY,
                market_id TEXT NOT NULL,
                token_id TEXT NOT NULL,
                side TEXT NOT NULL,
                size TEXT NOT NULL,
                entry_price TEXT NOT NULL,
                entry_time INTEGER NOT NULL,
                exit_price TEXT,
                exit_time INTEGER,
                pnl TEXT,
                status TEXT NOT NULL,
                paper INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                buy_order_id TEXT,
                sell_order_id TEXT,
                strategy TEXT NOT NULL DEFAULT 'draw_time_decay'
            )",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_trade(&self, trade: &Trade) -> Result<()> {
        sqlx::query(
            "INSERT INTO trades (id, market_id, token_id, side, size, entry_price, entry_time, exit_price, exit_time, pnl, status, paper, created_at, updated_at, buy_order_id, sell_order_id, strategy)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&trade.id)
        .bind(&trade.market_id)
        .bind(&trade.token_id)
        .bind(trade.side_str())
        .bind(trade.size.to_string())
        .bind(trade.entry_price.to_string())
        .bind(trade.entry_time)
        .bind(trade.exit_price.map(|p| p.to_string()))
        .bind(trade.exit_time)
        .bind(trade.pnl.map(|p| p.to_string()))
        .bind(trade.status.to_string())
        .bind(trade.paper as i32)
        .bind(trade.created_at)
        .bind(trade.updated_at)
        .bind(&trade.buy_order_id)
        .bind(&trade.sell_order_id)
        .bind(trade.strategy.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_trade(&self, trade_id: &str) -> Result<Option<Trade>> {
        let row = sqlx::query("SELECT * FROM trades WHERE id = ?")
            .bind(trade_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| Trade::from_row(&r)).transpose()?)
    }

    pub async fn update_trade(&self, trade: &Trade) -> Result<()> {
        sqlx::query(
            "UPDATE trades SET exit_price = ?, exit_time = ?, pnl = ?, status = ?, updated_at = ?, sell_order_id = ? WHERE id = ?",
        )
        .bind(trade.exit_price.map(|p| p.to_string()))
        .bind(trade.exit_time)
        .bind(trade.pnl.map(|p| p.to_string()))
        .bind(trade.status.to_string())
        .bind(trade.updated_at)
        .bind(&trade.sell_order_id)
        .bind(&trade.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_open_trades(&self) -> Result<Vec<Trade>> {
        let rows = sqlx::query("SELECT * FROM trades WHERE status = 'open'")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .iter()
            .map(|r| Trade::from_row(r))
            .collect::<Result<Vec<_>>>()?)
    }
}

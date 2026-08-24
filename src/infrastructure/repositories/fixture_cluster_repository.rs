use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};

use crate::{
    domain::entities::{FixtureCluster, MarketType, Outcome},
    shared::error::Result,
};

use super::game_repository::GameRepository;

#[derive(Debug)]
pub struct FixtureClusterRepository {
    pool: SqlitePool,
    game_repository: Arc<GameRepository>,
}

impl FixtureClusterRepository {
    pub fn from_pool(pool: SqlitePool, game_repository: Arc<GameRepository>) -> Self {
        Self {
            game_repository,
            pool,
        }
    }

    pub async fn run_migrations(&self) -> Result<()> {
        self.game_repository.run_migrations().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS fixture_cluster (
                key TEXT PRIMARY KEY,
                representative_game_id TEXT REFERENCES games(id) ON DELETE SET NULL,
                game_date INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                closed INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&self.pool)
        .await?;

        // Databases created before the closed flag predate the column above.
        if let Err(e) =
            sqlx::query("ALTER TABLE fixture_cluster ADD COLUMN closed INTEGER NOT NULL DEFAULT 0")
                .execute(&self.pool)
                .await
        {
            if !e.to_string().contains("duplicate column name") {
                return Err(e.into());
            }
        }

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS fixture_cluster_game (
                fixture_key TEXT NOT NULL REFERENCES fixture_cluster(key) ON DELETE CASCADE,
                game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
                PRIMARY KEY (fixture_key, game_id)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_fixture_cluster_game_game ON fixture_cluster_game(game_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS fixture_cluster_diff (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                fixture_key TEXT NOT NULL REFERENCES fixture_cluster(key) ON DELETE CASCADE,
                market_type TEXT NOT NULL,
                outcome TEXT NOT NULL,
                diff REAL NOT NULL,
                created_at INTEGER NOT NULL,
                UNIQUE(fixture_key, market_type, outcome)
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_cluster(&self, cluster: &FixtureCluster) -> Result<()> {
        for game in cluster.games() {
            if self.game_repository.game_exists(&game.id).await? {
                self.game_repository.update_game(game).await?;
            } else {
                self.game_repository.insert_game(game).await?;
            }
        }

        let now = Utc::now().timestamp();
        let game_date = cluster
            .representative_game()
            .or_else(|| cluster.games().next())
            .map(|g| g.date.and_utc().timestamp())
            .ok_or_else(|| "cluster has no games".to_string())?;

        // Atomic rebuild: concurrent persists of the same key serialize on the
        // write lock, so one task can never insert rows the other just deleted.
        //
        // BEGIN IMMEDIATE takes the write lock up front: with concurrent writers
        // (other clusters, draw-trade bot), a deferred transaction that reads
        // before writing would fail with SQLITE_BUSY_SNAPSHOT, which
        // busy_timeout does not retry.
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        sqlx::query(
            "INSERT INTO fixture_cluster (key, representative_game_id, game_date, created_at, updated_at, closed)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET
                 representative_game_id = excluded.representative_game_id,
                 game_date = excluded.game_date,
                 updated_at = excluded.updated_at,
                 closed = excluded.closed",
        )
        .bind(cluster.key())
        .bind(cluster.representative_game().map(|g| g.id.clone()))
        .bind(game_date)
        .bind(now)
        .bind(cluster.updated_at().timestamp())
        .bind(i64::from(cluster.is_closed()))
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM fixture_cluster_game WHERE fixture_key = ?")
            .bind(cluster.key())
            .execute(&mut *tx)
            .await?;

        for game in cluster.games() {
            sqlx::query("INSERT INTO fixture_cluster_game (fixture_key, game_id) VALUES (?, ?)")
                .bind(cluster.key())
                .bind(&game.id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn insert_cluster_diffs(
        &self,
        fixture_key: &str,
        diffs: &HashMap<(MarketType, Outcome), f64>,
    ) -> Result<()> {
        let now = Utc::now().timestamp();
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;

        for ((market_type, outcome), diff) in diffs {
            sqlx::query(
                "INSERT INTO fixture_cluster_diff (fixture_key, market_type, outcome, diff, created_at)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(fixture_key, market_type, outcome) DO UPDATE SET
                     diff = excluded.diff,
                     created_at = excluded.created_at",
            )
            .bind(fixture_key)
            .bind(market_type.to_key_string())
            .bind(format!("{:?}", outcome))
            .bind(diff)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn get_cluster_diffs(&self, key: &str) -> Result<HashMap<(MarketType, Outcome), f64>> {
        let rows = sqlx::query(
            "SELECT market_type, outcome, diff FROM fixture_cluster_diff WHERE fixture_key = ?",
        )
        .bind(key)
        .fetch_all(&self.pool)
        .await?;

        let mut diffs = HashMap::with_capacity(rows.len());
        for row in rows {
            match parse_diff_row(&row)? {
                Some((market_type, outcome, diff)) => {
                    diffs.insert((market_type, outcome), diff);
                }
                None => {
                    tracing::warn!(
                        fixture = key,
                        "skipping unknown market_type/outcome in fixture_cluster_diff"
                    );
                }
            }
        }

        Ok(diffs)
    }

    pub async fn get_all_cluster_diffs(&self) -> Result<Vec<(MarketType, Outcome, f64)>> {
        let rows = sqlx::query("SELECT market_type, outcome, diff FROM fixture_cluster_diff")
            .fetch_all(&self.pool)
            .await?;

        let mut diffs = Vec::with_capacity(rows.len());
        for row in rows {
            match parse_diff_row(&row)? {
                Some(diff) => diffs.push(diff),
                None => {
                    tracing::warn!("skipping unknown market_type/outcome in fixture_cluster_diff");
                }
            }
        }

        Ok(diffs)
    }

    pub async fn get_cluster(&self, key: &str) -> Result<Option<FixtureCluster>> {
        let row = sqlx::query("SELECT key, updated_at, closed FROM fixture_cluster WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let updated_at_ts: i64 = row.try_get("updated_at")?;
        let updated_at = DateTime::from_timestamp(updated_at_ts, 0)
            .ok_or_else(|| format!("Invalid updated_at timestamp for cluster '{}'", key))?;
        let closed: i64 = row.try_get("closed")?;

        let game_ids: Vec<String> = sqlx::query(
            "SELECT game_id FROM fixture_cluster_game WHERE fixture_key = ? ORDER BY game_id",
        )
        .bind(key)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| row.try_get("game_id"))
        .collect::<sqlx::Result<Vec<_>>>()?;

        let mut games = Vec::with_capacity(game_ids.len());
        for game_id in game_ids {
            if let Some(game) = self.game_repository.get_game(&game_id).await? {
                games.push(game);
            }
        }

        let mean_diffs = self.get_cluster_diffs(key).await?;

        Ok(Some(FixtureCluster::from_persisted(
            key.to_string(),
            games,
            updated_at,
            mean_diffs,
            closed != 0,
        )))
    }

    pub async fn get_all_clusters(&self) -> Result<Vec<FixtureCluster>> {
        let rows = sqlx::query("SELECT key FROM fixture_cluster ORDER BY updated_at DESC")
            .fetch_all(&self.pool)
            .await?;

        let keys: Vec<String> = rows
            .iter()
            .map(|row| row.try_get("key"))
            .collect::<sqlx::Result<Vec<_>>>()?;

        let mut clusters = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(cluster) = self.get_cluster(&key).await? {
                clusters.push(cluster);
            }
        }

        Ok(clusters)
    }

    pub async fn get_future_clusters(&self) -> Result<Vec<FixtureCluster>> {
        let now = Utc::now().timestamp();
        let rows =
            sqlx::query("SELECT key FROM fixture_cluster WHERE game_date > ? ORDER BY game_date")
                .bind(now)
                .fetch_all(&self.pool)
                .await?;

        let keys: Vec<String> = rows
            .iter()
            .map(|row| row.try_get("key"))
            .collect::<sqlx::Result<Vec<_>>>()?;

        let mut clusters = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(cluster) = self.get_cluster(&key).await? {
                clusters.push(cluster);
            }
        }

        Ok(clusters)
    }
}

fn parse_diff_row(row: &SqliteRow) -> Result<Option<(MarketType, Outcome, f64)>> {
    let market_type: String = row.try_get("market_type")?;
    let outcome: String = row.try_get("outcome")?;

    let Some(market_type) = MarketType::from_key_string(&market_type) else {
        return Ok(None);
    };
    let Some(outcome) = Outcome::from_key_string(&outcome) else {
        return Ok(None);
    };

    Ok(Some((market_type, outcome, row.try_get("diff")?)))
}

#[cfg(test)]
mod tests;

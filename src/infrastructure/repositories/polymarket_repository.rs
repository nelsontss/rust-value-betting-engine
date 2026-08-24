use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

use crate::shared::error::Result;

pub struct PolymarketRepository {
    pool: SqlitePool,
}

impl PolymarketRepository {
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS polymarket_events (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                slug TEXT,
                description TEXT,
                series_slug TEXT,
                home_team TEXT,
                away_team TEXT,
                tags TEXT,
                volume REAL,
                volume_24h REAL,
                closed INTEGER NOT NULL DEFAULT 1,
                start_date TEXT,
                end_date TEXT,
                url TEXT,
                fetched_at TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS polymarket_markets (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL,
                question TEXT NOT NULL,
                slug TEXT,
                description TEXT,
                status TEXT,
                market_type TEXT,
                resolution_date TEXT,
                volume REAL,
                volume_24h REAL,
                liquidity REAL,
                open_interest REAL,
                clob_token_id_yes TEXT,
                clob_token_id_no TEXT,
                outcome_price_yes REAL,
                outcome_price_no REAL,
                outcome_yes_label TEXT DEFAULT 'Yes',
                outcome_no_label TEXT DEFAULT 'No',
                url TEXT,
                FOREIGN KEY (event_id) REFERENCES polymarket_events(id)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS price_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                market_id TEXT NOT NULL,
                token_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                open REAL NOT NULL,
                high REAL NOT NULL,
                low REAL NOT NULL,
                close REAL NOT NULL,
                volume REAL NOT NULL,
                FOREIGN KEY (market_id) REFERENCES polymarket_markets(id)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_price_history_market_token
             ON price_history(market_id, token_id)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_price_history_timestamp ON price_history(timestamp)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_markets_event_id ON polymarket_markets(event_id)",
        )
        .execute(&self.pool)
        .await?;

        // v2: add derived_type column and backfill
        let result = sqlx::query("ALTER TABLE polymarket_markets ADD COLUMN derived_type TEXT")
            .execute(&self.pool)
            .await;
        if result.is_ok() {
            sqlx::query(
                "UPDATE polymarket_markets SET derived_type = 'draw' WHERE slug LIKE '%draw%'",
            )
            .execute(&self.pool)
            .await?;
        }

        // v3: add has_prices column
        let result = sqlx::query(
            "ALTER TABLE polymarket_markets ADD COLUMN has_prices INTEGER NOT NULL DEFAULT 0",
        )
        .execute(&self.pool)
        .await;
        if result.is_ok() {
            sqlx::query(
                "UPDATE polymarket_markets SET has_prices = 1
                 WHERE id IN (SELECT DISTINCT market_id FROM price_history)",
            )
            .execute(&self.pool)
            .await?;
        }

        // v4: add start_time column (actual match kickoff, not listing date)
        let _ = sqlx::query("ALTER TABLE polymarket_events ADD COLUMN start_time TEXT")
            .execute(&self.pool)
            .await;

        Ok(())
    }

    pub async fn insert_event(
        &self,
        id: &str,
        title: &str,
        slug: Option<&str>,
        description: Option<&str>,
        series_slug: Option<&str>,
        home_team: Option<&str>,
        away_team: Option<&str>,
        tags_json: Option<&str>,
        volume: Option<f64>,
        volume_24h: Option<f64>,
        closed: bool,
        start_date: Option<&str>,
        start_time: Option<&str>,
        end_date: Option<&str>,
        fetched_at: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO polymarket_events
             (id, title, slug, description, series_slug, home_team, away_team,
              tags, volume, volume_24h, closed, start_date, start_time, end_date, fetched_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(title)
        .bind(slug)
        .bind(description)
        .bind(series_slug)
        .bind(home_team)
        .bind(away_team)
        .bind(tags_json)
        .bind(volume)
        .bind(volume_24h)
        .bind(closed as i32)
        .bind(start_date)
        .bind(start_time)
        .bind(end_date)
        .bind(fetched_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_market(
        &self,
        id: &str,
        event_id: &str,
        question: &str,
        slug: Option<&str>,
        description: Option<&str>,
        status: Option<&str>,
        market_type: Option<&str>,
        volume: Option<f64>,
        volume_24h: Option<f64>,
        liquidity: Option<f64>,
        open_interest: Option<f64>,
        clob_token_id_yes: Option<&str>,
        clob_token_id_no: Option<&str>,
        outcome_price_yes: Option<f64>,
        outcome_price_no: Option<f64>,
    ) -> Result<()> {
        let derived = slug.filter(|s| s.contains("draw")).map(|_| "draw");

        sqlx::query(
            "INSERT OR REPLACE INTO polymarket_markets
             (id, event_id, question, slug, description, status, market_type,
              volume, volume_24h, liquidity, open_interest,
              clob_token_id_yes, clob_token_id_no,
              outcome_price_yes, outcome_price_no, has_prices, derived_type)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?)",
        )
        .bind(id)
        .bind(event_id)
        .bind(question)
        .bind(slug)
        .bind(description)
        .bind(status)
        .bind(market_type)
        .bind(volume)
        .bind(volume_24h)
        .bind(liquidity)
        .bind(open_interest)
        .bind(clob_token_id_yes)
        .bind(clob_token_id_no)
        .bind(outcome_price_yes)
        .bind(outcome_price_no)
        .bind(derived)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_price_candle(
        &self,
        market_id: &str,
        token_id: &str,
        timestamp: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO price_history
             (market_id, token_id, timestamp, open, high, low, close, volume)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(market_id)
        .bind(token_id)
        .bind(timestamp)
        .bind(open)
        .bind(high)
        .bind(low)
        .bind(close)
        .bind(volume)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_market_has_prices(&self, market_id: &str) -> Result<()> {
        sqlx::query("UPDATE polymarket_markets SET has_prices = 1 WHERE id = ?")
            .bind(market_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_markets_without_prices(&self) -> Result<Vec<MarketRow>> {
        let rows = sqlx::query(
            "SELECT m.id, m.event_id, m.question, m.clob_token_id_yes, m.clob_token_id_no, e.start_time, e.end_date
             FROM polymarket_markets m
             LEFT JOIN polymarket_events e ON e.id = m.event_id
             WHERE (m.clob_token_id_yes IS NOT NULL OR m.clob_token_id_no IS NOT NULL)
               AND m.has_prices = 0
             ORDER BY m.derived_type = 'draw' DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(market_row_from).collect()
    }

    pub async fn get_events_in_date_range(
        &self,
        from: &str,
        to: &str,
        order: &str,
    ) -> Result<Vec<EventRow>> {
        let order = if order == "DESC" { "DESC" } else { "ASC" };
        let rows = if order == "DESC" {
            sqlx::query(
                "SELECT e.id, e.title, e.home_team, e.away_team, e.start_date, e.start_time, e.end_date
                 FROM polymarket_events e
                 WHERE e.start_date >= ? AND e.start_date <= ?
                 ORDER BY e.start_date DESC",
            )
        } else {
            sqlx::query(
                "SELECT e.id, e.title, e.home_team, e.away_team, e.start_date, e.start_time, e.end_date
                 FROM polymarket_events e
                 WHERE e.start_date >= ? AND e.start_date <= ?
                 ORDER BY e.start_date ASC",
            )
        }
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(event_row_from).collect()
    }

    pub async fn get_markets_for_event(&self, event_id: &str) -> Result<Vec<MarketRow>> {
        let rows = sqlx::query(
            "SELECT m.id, m.event_id, m.question, m.clob_token_id_yes, m.clob_token_id_no, e.start_time, e.end_date
             FROM polymarket_markets m
             LEFT JOIN polymarket_events e ON e.id = m.event_id
             WHERE m.event_id = ?",
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(market_row_from).collect()
    }

    pub async fn event_count(&self) -> Result<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM polymarket_events")
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn price_count(&self, market_id: &str) -> Result<i64> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM price_history WHERE market_id = ?")
                .bind(market_id)
                .fetch_one(&self.pool)
                .await?,
        )
    }

    pub async fn get_draw_markets(&self) -> Result<Vec<MarketRow>> {
        let rows = sqlx::query(
            "SELECT m.id, m.event_id, m.question, m.clob_token_id_yes, m.clob_token_id_no, e.start_time, e.end_date
             FROM polymarket_markets m
             LEFT JOIN polymarket_events e ON e.id = m.event_id
             WHERE m.derived_type = 'draw'
             ORDER BY e.start_time ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(market_row_from).collect()
    }

    pub async fn get_candles_for_market(&self, market_id: &str) -> Result<Vec<PriceCandleRow>> {
        let rows = sqlx::query(
            "SELECT timestamp, open, high, low, close, volume
             FROM price_history
             WHERE market_id = ?
             ORDER BY timestamp ASC",
        )
        .bind(market_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(candle_row_from).collect()
    }
}

#[derive(Debug)]
pub struct MarketRow {
    pub id: String,
    pub event_id: String,
    pub question: Option<String>,
    pub clob_token_id_yes: Option<String>,
    pub clob_token_id_no: Option<String>,
    pub match_start: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug)]
pub struct EventRow {
    pub id: String,
    pub title: Option<String>,
    pub home_team: Option<String>,
    pub away_team: Option<String>,
    pub start_date: Option<String>,
    pub start_time: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug)]
pub struct PriceCandleRow {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

fn market_row_from(row: &SqliteRow) -> Result<MarketRow> {
    Ok(MarketRow {
        id: row.try_get("id")?,
        event_id: row.try_get("event_id")?,
        question: row.try_get("question")?,
        clob_token_id_yes: row.try_get("clob_token_id_yes")?,
        clob_token_id_no: row.try_get("clob_token_id_no")?,
        match_start: row.try_get("start_time")?,
        end_date: row.try_get("end_date")?,
    })
}

fn event_row_from(row: &SqliteRow) -> Result<EventRow> {
    Ok(EventRow {
        id: row.try_get("id")?,
        title: row.try_get("title")?,
        home_team: row.try_get("home_team")?,
        away_team: row.try_get("away_team")?,
        start_date: row.try_get("start_date")?,
        start_time: row.try_get("start_time")?,
        end_date: row.try_get("end_date")?,
    })
}

fn candle_row_from(row: &SqliteRow) -> Result<PriceCandleRow> {
    Ok(PriceCandleRow {
        timestamp: row.try_get("timestamp")?,
        open: row.try_get("open")?,
        high: row.try_get("high")?,
        low: row.try_get("low")?,
        close: row.try_get("close")?,
        volume: row.try_get("volume")?,
    })
}

#[cfg(test)]
mod tests {
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
}

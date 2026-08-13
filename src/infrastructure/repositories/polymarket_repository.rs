use turso::{Builder, Connection, Value, params_from_iter};

pub struct PolymarketRepository {
    conn: Connection,
}

impl PolymarketRepository {
    pub async fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Builder::new_local(db_path)
            .build()
            .await?;
        let conn = db.connect()?;
        Ok(Self { conn })
    }

    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn
            .execute(
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
                (),
            )
            .await?;

        self.conn
            .execute(
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
                (),
            )
            .await?;

        self.conn
            .execute(
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
                (),
            )
            .await?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_price_history_market_token
             ON price_history(market_id, token_id)",
                (),
            )
            .await?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_price_history_timestamp
             ON price_history(timestamp)",
                (),
            )
            .await?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_markets_event_id
             ON polymarket_markets(event_id)",
                (),
            )
            .await?;

        // v2: add derived_type column and backfill
        let result = self
            .conn
            .execute("ALTER TABLE polymarket_markets ADD COLUMN derived_type TEXT", ())
            .await;
        if result.is_ok() {
            self.conn
                .execute(
                    "UPDATE polymarket_markets SET derived_type = 'draw' WHERE slug LIKE '%draw%'",
                    (),
                )
                .await?;
        }

        // v3: add has_prices column
        let result = self
            .conn
            .execute(
                "ALTER TABLE polymarket_markets ADD COLUMN has_prices INTEGER NOT NULL DEFAULT 0",
                (),
            )
            .await;
        if result.is_ok() {
            self.conn
                .execute(
                    "UPDATE polymarket_markets SET has_prices = 1 WHERE id IN (SELECT DISTINCT market_id FROM price_history)",
                    (),
                )
                .await?;
        }

        // v4: add start_time column (actual match kickoff, not listing date)
        let _ = self
            .conn
            .execute("ALTER TABLE polymarket_events ADD COLUMN start_time TEXT", ())
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO polymarket_events
             (id, title, slug, description, series_slug, home_team, away_team,
              tags, volume, volume_24h, closed, start_date, start_time, end_date, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params_from_iter([
                    some_str(id),
                    some_str(title),
                    opt_str(slug),
                    opt_str(description),
                    opt_str(series_slug),
                    opt_str(home_team),
                    opt_str(away_team),
                    opt_str(tags_json),
                    opt_f64(volume),
                    opt_f64(volume_24h),
                    Value::Integer(closed as i64),
                    opt_str(start_date),
                    opt_str(start_time),
                    opt_str(end_date),
                    some_str(fetched_at),
                ]),
            )
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let derived = slug
            .filter(|s| s.contains("draw"))
            .map(|_| "draw");

        self.conn
            .execute(
                "INSERT OR REPLACE INTO polymarket_markets
             (id, event_id, question, slug, description, status, market_type,
              volume, volume_24h, liquidity, open_interest,
              clob_token_id_yes, clob_token_id_no,
              outcome_price_yes, outcome_price_no, has_prices, derived_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, 0, ?16)",
                params_from_iter([
                    some_str(id),
                    some_str(event_id),
                    some_str(question),
                    opt_str(slug),
                    opt_str(description),
                    opt_str(status),
                    opt_str(market_type),
                    opt_f64(volume),
                    opt_f64(volume_24h),
                    opt_f64(liquidity),
                    opt_f64(open_interest),
                    opt_str(clob_token_id_yes),
                    opt_str(clob_token_id_no),
                    opt_f64(outcome_price_yes),
                    opt_f64(outcome_price_no),
                    opt_str(derived),
                ]),
            )
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.conn
            .execute(
                "INSERT INTO price_history
             (market_id, token_id, timestamp, open, high, low, close, volume)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params_from_iter([
                    some_str(market_id),
                    some_str(token_id),
                    Value::Integer(timestamp),
                    Value::Real(open),
                    Value::Real(high),
                    Value::Real(low),
                    Value::Real(close),
                    Value::Real(volume),
                ]),
            )
            .await?;
        Ok(())
    }

    pub async fn set_market_has_prices(
        &self,
        market_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.conn
            .execute(
                "UPDATE polymarket_markets SET has_prices = 1 WHERE id = ?1",
                params_from_iter([some_str(market_id)]),
            )
            .await?;
        Ok(())
    }

    pub async fn get_markets_without_prices(
        &self,
    ) -> Result<Vec<MarketRow>, Box<dyn std::error::Error>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT m.id, m.event_id, m.question, m.clob_token_id_yes, m.clob_token_id_no, e.start_time, e.end_date
             FROM polymarket_markets m
             LEFT JOIN polymarket_events e ON e.id = m.event_id
             WHERE (m.clob_token_id_yes IS NOT NULL OR m.clob_token_id_no IS NOT NULL)
               AND m.has_prices = 0
             ORDER BY m.derived_type = 'draw' DESC",
            )
            .await?;

        let mut rows = stmt.query(()).await?;
        let mut result = Vec::new();

        while let Some(row) = rows.next().await? {
            result.push(MarketRow {
                id: row.get_text(0)?,
                event_id: row.get_text(1)?,
                question: row.get_opt_text(2)?,
                clob_token_id_yes: row.get_opt_text(3)?,
                clob_token_id_no: row.get_opt_text(4)?,
                match_start: row.get_opt_text(5)?,
                end_date: row.get_opt_text(6)?,
            });
        }

        Ok(result)
    }

    pub async fn get_events_in_date_range(
        &self,
        from: &str,
        to: &str,
        order: &str,
    ) -> Result<Vec<EventRow>, Box<dyn std::error::Error>> {
        let order = if order == "DESC" { "DESC" } else { "ASC" };
        let sql = format!(
            "SELECT e.id, e.title, e.home_team, e.away_team, e.start_date, e.start_time, e.end_date
             FROM polymarket_events e
             WHERE e.start_date >= ?1 AND e.start_date <= ?2
             ORDER BY e.start_date {}",
            order
        );
        let mut stmt = self.conn.prepare(&sql).await?;

        let mut rows = stmt
            .query(params_from_iter([some_str(from), some_str(to)]))
            .await?;
        let mut result = Vec::new();

        while let Some(row) = rows.next().await? {
            result.push(EventRow {
                id: row.get_text(0)?,
                title: row.get_opt_text(1)?,
                home_team: row.get_opt_text(2)?,
                away_team: row.get_opt_text(3)?,
                start_date: row.get_opt_text(4)?,
                start_time: row.get_opt_text(5)?,
                end_date: row.get_opt_text(6)?,
            });
        }

        Ok(result)
    }

    pub async fn get_markets_for_event(
        &self,
        event_id: &str,
    ) -> Result<Vec<MarketRow>, Box<dyn std::error::Error>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT m.id, m.event_id, m.question, m.clob_token_id_yes, m.clob_token_id_no, e.start_time, e.end_date
             FROM polymarket_markets m
             LEFT JOIN polymarket_events e ON e.id = m.event_id
             WHERE m.event_id = ?1",
            )
            .await?;

        let mut rows = stmt
            .query(params_from_iter([some_str(event_id)]))
            .await?;
        let mut result = Vec::new();

        while let Some(row) = rows.next().await? {
            result.push(MarketRow {
                id: row.get_text(0)?,
                event_id: row.get_text(1)?,
                question: row.get_opt_text(2)?,
                clob_token_id_yes: row.get_opt_text(3)?,
                clob_token_id_no: row.get_opt_text(4)?,
                match_start: row.get_opt_text(5)?,
                end_date: row.get_opt_text(6)?,
            });
        }

        Ok(result)
    }

    pub async fn event_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM polymarket_events")
            .await?;

        let mut rows = stmt.query(()).await?;
        let row = rows
            .next()
            .await?
            .ok_or("no rows from count query")?;
        Ok(row.get_i64(0)?)
    }

    pub async fn price_count(&self, market_id: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM price_history WHERE market_id = ?1")
            .await?;

        let mut rows = stmt
            .query(params_from_iter([some_str(market_id)]))
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or("no rows from count query")?;
        Ok(row.get_i64(0)?)
    }

    pub async fn get_draw_markets(&self) -> Result<Vec<MarketRow>, Box<dyn std::error::Error>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT m.id, m.event_id, m.question, m.clob_token_id_yes, m.clob_token_id_no, e.start_time, e.end_date
             FROM polymarket_markets m
             LEFT JOIN polymarket_events e ON e.id = m.event_id
             WHERE m.derived_type = 'draw'
             ORDER BY e.start_time ASC",
            )
            .await?;

        let mut rows = stmt.query(()).await?;
        let mut result = Vec::new();

        while let Some(row) = rows.next().await? {
            result.push(MarketRow {
                id: row.get_text(0)?,
                event_id: row.get_text(1)?,
                question: row.get_opt_text(2)?,
                clob_token_id_yes: row.get_opt_text(3)?,
                clob_token_id_no: row.get_opt_text(4)?,
                match_start: row.get_opt_text(5)?,
                end_date: row.get_opt_text(6)?,
            });
        }

        Ok(result)
    }

    pub async fn get_candles_for_market(
        &self,
        market_id: &str,
    ) -> Result<Vec<PriceCandleRow>, Box<dyn std::error::Error>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT timestamp, open, high, low, close, volume
             FROM price_history
             WHERE market_id = ?1
             ORDER BY timestamp ASC",
            )
            .await?;

        let mut rows = stmt
            .query(params_from_iter([some_str(market_id)]))
            .await?;
        let mut result = Vec::new();

        while let Some(row) = rows.next().await? {
            result.push(PriceCandleRow {
                timestamp: row.get_i64(0)?,
                open: row.get_f64(1)?,
                high: row.get_f64(2)?,
                low: row.get_f64(3)?,
                close: row.get_f64(4)?,
                volume: row.get_f64(5)?,
            });
        }

        Ok(result)
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

// -- turso value helpers --

fn some_str(s: &str) -> Value {
    Value::Text(s.to_string())
}

fn opt_str(s: Option<&str>) -> Value {
    s.map(|s| Value::Text(s.to_string())).unwrap_or(Value::Null)
}

fn opt_f64(v: Option<f64>) -> Value {
    v.map(Value::Real).unwrap_or(Value::Null)
}

// -- turso Row helpers --

trait RowExt {
    fn get_text(&self, idx: usize) -> Result<String, Box<dyn std::error::Error>>;
    fn get_i64(&self, idx: usize) -> Result<i64, Box<dyn std::error::Error>>;
    fn get_f64(&self, idx: usize) -> Result<f64, Box<dyn std::error::Error>>;
    fn get_opt_text(&self, idx: usize) -> Result<Option<String>, Box<dyn std::error::Error>>;
}

impl RowExt for turso::Row {
    fn get_text(&self, idx: usize) -> Result<String, Box<dyn std::error::Error>> {
        match self.get_value(idx)? {
            Value::Text(s) => Ok(s),
            _ => Err("expected text".into()),
        }
    }

    fn get_i64(&self, idx: usize) -> Result<i64, Box<dyn std::error::Error>> {
        match self.get_value(idx)? {
            Value::Integer(n) => Ok(n),
            _ => Err("expected integer".into()),
        }
    }

    fn get_f64(&self, idx: usize) -> Result<f64, Box<dyn std::error::Error>> {
        match self.get_value(idx)? {
            Value::Real(n) => Ok(n),
            Value::Integer(n) => Ok(n as f64),
            _ => Err("expected real".into()),
        }
    }

    fn get_opt_text(&self, idx: usize) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match self.get_value(idx)? {
            Value::Text(s) => Ok(Some(s)),
            Value::Null => Ok(None),
            _ => Err("expected text or null".into()),
        }
    }
}

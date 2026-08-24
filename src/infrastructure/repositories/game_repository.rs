use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

use crate::{
    domain::{
        Game, Market,
        entities::{MarketDataPoint, Platform},
    },
    shared::error::Result,
};

#[derive(Debug)]
pub struct GameRepository {
    pool: SqlitePool,
}

impl GameRepository {
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn game_exists(&self, game_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM games WHERE id = ?")
            .bind(game_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count > 0)
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS games (
                id TEXT PRIMARY KEY,
                home_team TEXT NOT NULL,
                away_team TEXT NOT NULL,
                country TEXT NOT NULL,
                competition TEXT NOT NULL,
                platform TEXT NOT NULL,
                date INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS markets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                market_id TEXT NOT NULL,
                game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
                market_type TEXT NOT NULL,
                line REAL,
                home REAL,
                draw REAL,
                away REAL,
                over REAL,
                under REAL,
                home_or_draw REAL,
                home_or_away REAL,
                draw_or_away REAL,
                is_last_market INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_markets_game_last ON markets(game_id, is_last_market)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_markets_game_created ON markets(game_id, created_at, id)",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_game(&self, game: &Game) -> Result<()> {
        let now = Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO games (id, home_team, away_team, country, competition, platform, date, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&game.id)
        .bind(game.home_team())
        .bind(game.away_team())
        .bind(game.country())
        .bind(game.competition())
        .bind(platform_to_string(game.platform()))
        .bind(game.date.and_utc().timestamp())
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.insert_markets(game).await?;

        Ok(())
    }

    pub async fn get_game(&self, game_id: &str) -> Result<Option<Game>> {
        let row = sqlx::query("SELECT * FROM games WHERE id = ?")
            .bind(game_id)
            .fetch_optional(&self.pool)
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let markets = self.get_markets(game_id).await?;
        let date_ts: i64 = row.try_get("date")?;
        let date = DateTime::from_timestamp(date_ts, 0)
            .map(|dt| dt.naive_utc())
            .ok_or_else(|| format!("Invalid date timestamp for game '{}'", game_id))?;

        let game = Game::new_with_id(
            &row.try_get::<String, _>("id")?,
            &row.try_get::<String, _>("home_team")?,
            &row.try_get::<String, _>("away_team")?,
            &row.try_get::<String, _>("country")?,
            &row.try_get::<String, _>("competition")?,
            date,
            platform_from_string(&row.try_get::<String, _>("platform")?)?,
            markets,
        );

        Ok(Some(game))
    }

    pub async fn update_game(&self, game: &Game) -> Result<()> {
        let now = Utc::now().timestamp();
        sqlx::query(
            "UPDATE games SET home_team = ?, away_team = ?, country = ?, competition = ?, platform = ?, date = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(game.home_team())
        .bind(game.away_team())
        .bind(game.country())
        .bind(game.competition())
        .bind(platform_to_string(game.platform()))
        .bind(game.date.and_utc().timestamp())
        .bind(now)
        .bind(&game.id)
        .execute(&self.pool)
        .await?;

        self.insert_markets(game).await?;

        Ok(())
    }

    async fn insert_markets(&self, game: &Game) -> Result<()> {
        let now = Utc::now().timestamp();
        let mut tx = self
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await?;

        let last_markets = last_market_columns(&mut *tx, &game.id).await?;

        for market in game.markets().values() {
            let cols = market_columns(market);
            let key = market_key(&cols);

            if last_markets.get(&key).is_some_and(|prev| *prev == cols) {
                continue;
            }

            sqlx::query(
                "UPDATE markets SET is_last_market = 0 WHERE game_id = ? AND market_id = ? AND market_type = ? AND line IS ? AND is_last_market = 1",
            )
            .bind(&game.id)
            .bind(&cols.id)
            .bind(&cols.market_type)
            .bind(cols.line)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                "INSERT INTO markets
                 (market_id, game_id, market_type, line, home, draw, away, over, under, home_or_draw, home_or_away, draw_or_away, is_last_market, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)",
            )
            .bind(&cols.id)
            .bind(&game.id)
            .bind(&cols.market_type)
            .bind(cols.line)
            .bind(cols.home)
            .bind(cols.draw)
            .bind(cols.away)
            .bind(cols.over)
            .bind(cols.under)
            .bind(cols.home_or_draw)
            .bind(cols.home_or_away)
            .bind(cols.draw_or_away)
            .bind(now)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        }

        let current_keys: std::collections::HashSet<MarketKey> =
            game.markets().values().map(|m| market_key(&market_columns(m))).collect();

        for (key, _) in last_markets.iter() {
            if !current_keys.contains(key) {
                sqlx::query(
                    "UPDATE markets SET is_last_market = 0 WHERE game_id = ? AND market_id = ? AND market_type = ? AND line IS ? AND is_last_market = 1",
                )
                .bind(&game.id)
                .bind(&key.0)
                .bind(&key.1)
                .bind(key.2)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn get_markets(&self, game_id: &str) -> Result<Vec<Market>> {        let rows = sqlx::query(
            "SELECT * FROM markets WHERE game_id = ? AND is_last_market = 1 ORDER BY market_type",
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(market_from_row).collect::<Result<Vec<_>>>()
    }

    pub async fn get_game_markets_history(&self, game_id: &str) -> Result<Vec<MarketDataPoint>> {
        let rows = sqlx::query("SELECT * FROM markets WHERE game_id = ? ORDER BY created_at, id")
            .bind(game_id)
            .fetch_all(&self.pool)
            .await?;

        rows.iter()
            .map(|row| {
                let market = market_from_row(row)?;
                let created_at: i64 = row.try_get("created_at")?;
                let datetime = DateTime::from_timestamp(created_at, 0).ok_or_else(|| {
                    format!("Invalid created_at timestamp for game '{}'", game_id)
                })?;

                Ok(MarketDataPoint::new_with_datetime(market, datetime))
            })
            .collect()
    }
}

#[derive(Default, PartialEq)]
struct MarketColumns {
    id: String,
    market_type: String,
    line: Option<f32>,
    home: Option<f64>,
    draw: Option<f64>,
    away: Option<f64>,
    over: Option<f64>,
    under: Option<f64>,
    home_or_draw: Option<f64>,
    home_or_away: Option<f64>,
    draw_or_away: Option<f64>,
}

fn market_id(market: &Market) -> String {
    match market {
        Market::MatchResult(m) => m.id.clone(),
        Market::Moneyline(m) => m.id(),
        Market::DoubleChance(m) => m.id(),
        Market::Total(m) => m.id(),
        Market::Handicap(m) => m.id(),
        Market::AsianHandicap(m) => m.id(),
    }
}

type MarketKey = (String, String, Option<u32>);

fn market_key(cols: &MarketColumns) -> MarketKey {
    (
        cols.id.clone(),
        cols.market_type.clone(),
        cols.line.map(f32::to_bits),
    )
}

async fn last_market_columns(
    executor: impl sqlx::SqliteExecutor<'_>,
    game_id: &str,
) -> Result<HashMap<MarketKey, MarketColumns>> {
    let rows = sqlx::query(
        "SELECT market_id, market_type, line, home, draw, away, over, under, home_or_draw, home_or_away, draw_or_away
         FROM markets WHERE game_id = ? AND is_last_market = 1",
    )
    .bind(game_id)
    .fetch_all(executor)
    .await?;

    let mut columns = HashMap::with_capacity(rows.len());

    for row in rows.iter() {
        let cols = MarketColumns {
            id: row.try_get("market_id")?,
            market_type: row.try_get("market_type")?,
            line: row.try_get("line")?,
            home: row.try_get("home")?,
            draw: row.try_get("draw")?,
            away: row.try_get("away")?,
            over: row.try_get("over")?,
            under: row.try_get("under")?,
            home_or_draw: row.try_get("home_or_draw")?,
            home_or_away: row.try_get("home_or_away")?,
            draw_or_away: row.try_get("draw_or_away")?,
        };
        columns.insert(market_key(&cols), cols);
    }

    Ok(columns)
}

fn market_columns(market: &Market) -> MarketColumns {
    match market {
        Market::MatchResult(m) => MarketColumns {
            id: market_id(market),
            market_type: "MatchResult".to_string(),
            home: Some(m.home.get()),
            draw: Some(m.draw.get()),
            away: Some(m.away.get()),
            ..MarketColumns::default()
        },
        Market::Moneyline(m) => MarketColumns {
            id: market_id(market),
            market_type: "Moneyline".to_string(),
            home: Some(m.home.get()),
            away: Some(m.away.get()),
            ..MarketColumns::default()
        },
        Market::DoubleChance(m) => MarketColumns {
            id: market_id(market),
            market_type: "DoubleChance".to_string(),
            home_or_draw: Some(m.home_or_draw.get()),
            home_or_away: Some(m.home_or_away.get()),
            draw_or_away: Some(m.draw_or_away.get()),
            ..MarketColumns::default()
        },
        Market::Total(m) => MarketColumns {
            id: market_id(market),
            market_type: "Total".to_string(),
            line: Some(m.line.0),
            over: Some(m.over.get()),
            under: Some(m.under.get()),
            ..MarketColumns::default()
        },
        Market::Handicap(m) => MarketColumns {
            id: market_id(market),
            market_type: "Handicap".to_string(),
            line: Some(m.line.0),
            home: Some(m.home.get()),
            draw: Some(m.draw.get()),
            away: Some(m.away.get()),
            ..MarketColumns::default()
        },
        Market::AsianHandicap(m) => MarketColumns {
            id: market_id(market),
            market_type: "AsianHandicap".to_string(),
            line: Some(m.line.0),
            home: Some(m.home.get()),
            away: Some(m.away.get()),
            ..MarketColumns::default()
        },
    }
}

fn market_from_row(row: &SqliteRow) -> Result<Market> {    let cols = MarketColumns {
        id: row.try_get("market_id")?,
        market_type: row.try_get("market_type")?,
        line: row.try_get("line")?,
        home: row.try_get("home")?,
        draw: row.try_get("draw")?,
        away: row.try_get("away")?,
        over: row.try_get("over")?,
        under: row.try_get("under")?,
        home_or_draw: row.try_get("home_or_draw")?,
        home_or_away: row.try_get("home_or_away")?,
        draw_or_away: row.try_get("draw_or_away")?,
    };

    market_from_columns(&cols)
}

fn market_from_columns(cols: &MarketColumns) -> Result<Market> {
    let odd_err = |id: &str| format!("invalid odds stored for market '{}'", id);

    match cols.market_type.as_str() {
        "MatchResult" => Ok(Market::match_result(
            &cols.id,
            cols.home.ok_or_else(|| odd_err(&cols.id))?,
            cols.draw.ok_or_else(|| odd_err(&cols.id))?,
            cols.away.ok_or_else(|| odd_err(&cols.id))?,
        )
        .map_err(|e| format!("{}: {:?}", odd_err(&cols.id), e))?),
        "Moneyline" => Ok(Market::moneyline(
            &cols.id,
            cols.home.ok_or_else(|| odd_err(&cols.id))?,
            cols.away.ok_or_else(|| odd_err(&cols.id))?,
        )
        .map_err(|e| format!("{}: {:?}", odd_err(&cols.id), e))?),
        "DoubleChance" => Ok(Market::double_chance(
            &cols.id,
            cols.home_or_draw.ok_or_else(|| odd_err(&cols.id))?,
            cols.home_or_away.ok_or_else(|| odd_err(&cols.id))?,
            cols.draw_or_away.ok_or_else(|| odd_err(&cols.id))?,
        )
        .map_err(|e| format!("{}: {:?}", odd_err(&cols.id), e))?),
        "Total" => Ok(Market::total(
            &cols.id,
            cols.line.ok_or_else(|| odd_err(&cols.id))?,
            cols.over.ok_or_else(|| odd_err(&cols.id))?,
            cols.under.ok_or_else(|| odd_err(&cols.id))?,
        )
        .map_err(|e| format!("{}: {:?}", odd_err(&cols.id), e))?),
        "Handicap" => Ok(Market::handicap(
            &cols.id,
            cols.line.ok_or_else(|| odd_err(&cols.id))?,
            cols.home.ok_or_else(|| odd_err(&cols.id))?,
            cols.draw.ok_or_else(|| odd_err(&cols.id))?,
            cols.away.ok_or_else(|| odd_err(&cols.id))?,
        )
        .map_err(|e| format!("{}: {:?}", odd_err(&cols.id), e))?),
        "AsianHandicap" => Ok(Market::asian_handicap(
            &cols.id,
            cols.line.ok_or_else(|| odd_err(&cols.id))?,
            cols.home.ok_or_else(|| odd_err(&cols.id))?,
            cols.away.ok_or_else(|| odd_err(&cols.id))?,
        )
        .map_err(|e| format!("{}: {:?}", odd_err(&cols.id), e))?),
        other => Err(format!("Unknown market type '{}' for market '{}'", other, cols.id).into()),
    }
}

fn platform_to_string(platform: Platform) -> &'static str {
    match platform {
        Platform::Betano => "betano",
        Platform::LeBull => "lebull",
        Platform::Bwin => "bwin",
        Platform::Polymarket => "polymarket",
    }
}

fn platform_from_string(value: &str) -> Result<Platform> {
    match value {
        "betano" => Ok(Platform::Betano),
        "lebull" => Ok(Platform::LeBull),
        "bwin" => Ok(Platform::Bwin),
        "polymarket" => Ok(Platform::Polymarket),
        _ => Err(format!("Invalid platform '{}'", value).into()),
    }
}

#[cfg(test)]
mod tests {
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
}

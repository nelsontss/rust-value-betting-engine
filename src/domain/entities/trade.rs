use chrono::Utc;
use polymarket_client_sdk_v2::clob::types::Side;
use polymarket_client_sdk_v2::types::Decimal;
use sqlx::Row;
use sqlx::sqlite::SqliteRow;
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(Clone, Debug, Display, EnumString, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum TradeStatus {
    Open,
    Closed,
}

#[derive(Clone, Debug, Display, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum TradeStrategy {
    DrawTimeDecay,
}

impl sqlx::Type<sqlx::Sqlite> for TradeStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for TradeStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<'r, sqlx::Sqlite>>::decode(value)?;
        s.parse::<TradeStatus>()
            .map_err(|e| format!("Invalid TradeStatus: {}", e).into())
    }
}

impl sqlx::Type<sqlx::Sqlite> for TradeStrategy {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for TradeStrategy {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<'r, sqlx::Sqlite>>::decode(value)?;
        s.parse::<TradeStrategy>()
            .map_err(|e| format!("Invalid TradeStrategy: {}", e).into())
    }
}

fn side_from_str(s: &str) -> Result<Side, sqlx::error::BoxDynError> {
    match s {
        "buy" => Ok(Side::Buy),
        "sell" => Ok(Side::Sell),
        _ => Err(format!("Invalid TradeSide: {}", s).into()),
    }
}

fn side_to_string(side: &Side) -> String {
    match side {
        Side::Buy => "buy".to_string(),
        Side::Sell => "sell".to_string(),
        _ => "unknown".to_string(),
    }
}

#[derive(Clone, Debug)]
pub struct Trade {
    pub id: String,
    pub market_id: String,
    pub token_id: String,
    pub side: Side,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub entry_time: i64,
    pub exit_price: Option<Decimal>,
    pub exit_time: Option<i64>,
    pub pnl: Option<Decimal>,
    pub status: TradeStatus,
    pub paper: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub buy_order_id: Option<String>,
    pub sell_order_id: Option<String>,
    pub strategy: TradeStrategy,
}

impl Trade {
    pub fn from_row(row: &SqliteRow) -> Result<Self, sqlx::error::BoxDynError> {
        let parse_decimal = |val: &str| -> Result<Decimal, sqlx::error::BoxDynError> {
            val.parse()
                .map_err(|e| format!("Invalid decimal '{}': {}", val, e).into())
        };

        Ok(Trade {
            id: row.try_get("id")?,
            market_id: row.try_get("market_id")?,
            token_id: row.try_get("token_id")?,
            side: side_from_str(&row.try_get::<String, _>("side")?)?,
            size: parse_decimal(&row.try_get::<String, _>("size")?)?,
            entry_price: parse_decimal(&row.try_get::<String, _>("entry_price")?)?,
            entry_time: row.try_get("entry_time")?,
            exit_price: row
                .try_get::<Option<String>, _>("exit_price")?
                .map(|v| parse_decimal(&v))
                .transpose()?,
            exit_time: row.try_get("exit_time")?,
            pnl: row
                .try_get::<Option<String>, _>("pnl")?
                .map(|v| parse_decimal(&v))
                .transpose()?,
            status: row.try_get("status")?,
            paper: row.try_get::<i32, _>("paper")? != 0,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            buy_order_id: row.try_get("buy_order_id")?,
            sell_order_id: row.try_get("sell_order_id")?,
            strategy: row.try_get("strategy")?,
        })
    }

    pub fn open_trade(
        order_id: Option<String>,
        market_id: String,
        token_id: String,
        side: Side,
        size: Decimal,
        entry_price: Decimal,
        entry_time: i64,
        paper: bool,
        strategy: TradeStrategy,
    ) -> Trade {
        let now = Utc::now().timestamp();
        Trade {
            id: Uuid::new_v4().to_string(),
            market_id,
            token_id,
            side,
            size,
            entry_price,
            entry_time,
            exit_price: None,
            exit_time: None,
            pnl: None,
            status: TradeStatus::Open,
            paper,
            created_at: now,
            updated_at: now,
            buy_order_id: order_id,
            sell_order_id: None,
            strategy,
        }
    }

    pub fn close_trade(&mut self, exit_price: Decimal, exit_time: i64, order_id: Option<String>) {
        self.exit_price = Some(exit_price);
        self.exit_time = Some(exit_time);
        self.pnl = Some((exit_price - self.entry_price) * self.size);
        self.status = TradeStatus::Closed;
        self.updated_at = Utc::now().timestamp();
        self.sell_order_id = order_id;
    }

    pub fn side_str(&self) -> String {
        side_to_string(&self.side)
    }
}

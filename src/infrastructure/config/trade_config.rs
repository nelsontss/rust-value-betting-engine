use chrono::Duration;
use polymarket_client_sdk_v2::types::{Decimal, dec};

pub struct TradeConfig {}

impl TradeConfig {
    pub const BANKROLL: Decimal = dec!(200);
    pub const MAX_VOLUME: Decimal = dec!(15000);
    pub const MAX_PRICE: Decimal = dec!(0.18);
    pub const MIN_PRICE: Decimal = dec!(0.03);
    pub const BUY_OFFSET: Duration = Duration::minutes(10);
    pub const SELL_OFFSET: Duration = Duration::minutes(10);
}

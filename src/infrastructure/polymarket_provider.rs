use std::str::FromStr as _;
use std::time::Duration;

use alloy::signers::k256::ecdsa::SigningKey;
use chrono::Utc;
use polymarket_client_sdk_v2::auth::state::Authenticated;
use polymarket_client_sdk_v2::auth::{LocalSigner, Normal, Signer as _};
use polymarket_client_sdk_v2::clob::types::request::PriceRequest;
use polymarket_client_sdk_v2::clob::types::response::CancelOrdersResponse;
use polymarket_client_sdk_v2::clob::types::{OrderStatusType, Side};
use polymarket_client_sdk_v2::gamma::types::request::EventsRequest;
use polymarket_client_sdk_v2::gamma::types::response::Market;
use polymarket_client_sdk_v2::types::{Decimal, U256};
use polymarket_client_sdk_v2::{
    POLYGON, PRIVATE_KEY_VAR,
    clob::{Client, Config},
};
use tokio::time::sleep;
use tracing::warn;

use crate::domain::entities::trade::{Trade, TradeStrategy};
use crate::infrastructure::config::trade_config::TradeConfig;
use crate::infrastructure::repositories::trade_repository::TradeRepository;
use crate::shared::error::Result;
use sqlx::SqlitePool;

pub struct PolymarketProvider {
    clob_client: Client<Authenticated<Normal>>,
    signer: LocalSigner<SigningKey>,
    trade_repository: TradeRepository,
    gamma_client: polymarket_client_sdk_v2::gamma::Client,
}

impl PolymarketProvider {
    pub async fn new(pool: SqlitePool) -> Result<Self> {
        let key: String = std::env::var(PRIVATE_KEY_VAR)?;
        let signer = LocalSigner::from_str(&key)?.with_chain_id(Some(POLYGON));

        let client = Client::new("https://clob.polymarket.com", Config::default())?
            .authentication_builder(&signer)
            .authenticate()
            .await?;

        Ok(PolymarketProvider {
            clob_client: client,
            signer,
            trade_repository: TradeRepository::from_pool(pool),
            gamma_client: polymarket_client_sdk_v2::gamma::Client::default(),
        })
    }

    pub async fn get_todays_draw_markets(&self) -> Result<Vec<Market>> {
        let events_request = &EventsRequest::builder()
            .tag_slug("soccer".to_string())
            .end_date_min(Utc::now())
            .end_date_max(Utc::now() + Duration::from_hours(24))
            .closed(false)
            .ascending(true)
            .build();
        let events = self.gamma_client.events(events_request).await?;

        let draw_markets: Vec<Market> = events
            .iter()
            .filter_map(|event| event.markets.as_ref())
            .flatten()
            .filter(|market| {
                market.slug.as_ref().is_none_or(|s| s.contains("draw"))
                    && market.volume < Some(TradeConfig::MAX_VOLUME)
            })
            .cloned()
            .collect();

        Ok(draw_markets)
    }

    pub async fn current_price(&self, token_id: U256) -> Result<Decimal> {
        let price_request = PriceRequest::builder()
            .token_id(token_id)
            .side(Side::Buy)
            .build();
        let result = self.clob_client.price(&price_request).await?;

        Ok(result.price)
    }

    async fn wait_for_fill(&self, order_id: &str) -> Result<()> {
        let timeout = Duration::from_secs(300);
        let sleep_duration = 200;
        let start = std::time::Instant::now();
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        loop {
            interval.tick().await;

            if start.elapsed() > timeout {
                return Err("Order not filled within timeout".into());
            }

            let order_response = match self.clob_client.order(order_id).await {
                Ok(o) => o,
                Err(e) => {
                    warn!("Poll failed: {}, retrying...", e);
                    continue;
                }
            };

            match order_response.status {
                OrderStatusType::Matched => return Ok(()),
                OrderStatusType::Canceled => {
                    return Err("Order cancelled/expired".into());
                }
                _ => {
                    sleep(Duration::from_millis(sleep_duration)).await;
                }
            }
        }
    }

    pub async fn place_trade_and_wait_for_resolution(
        &self,
        market_id: &str,
        token_id: &str,
        price: Decimal,
        size: Decimal,
        paper: bool,
        strategy: TradeStrategy,
    ) -> Result<Trade> {
        let order_id = if !paper {
            let order = self
                .clob_client
                .limit_order()
                .token_id(U256::from_str(token_id)?)
                .size(size.floor())
                .price(price)
                .side(Side::Buy)
                .post_only(true)
                .build()
                .await?;
            let signed_order = self.clob_client.sign(&self.signer, order).await?;
            let response = self.clob_client.post_order(signed_order).await?;

            Some(response.order_id.clone())
        } else {
            None
        };

        let trade = Trade::open_trade(
            order_id,
            market_id.to_string(),
            token_id.to_string(),
            Side::Buy,
            size,
            price,
            Utc::now().timestamp(),
            paper,
            strategy,
        );

        if !paper {
            self.wait_for_fill(&trade.id).await?;
        }

        self.trade_repository.insert_trade(&trade).await?;

        Ok(trade)
    }

    pub async fn exit_trade_and_wait_for_resolution(
        &self,
        trade_id: &str,
        price: Decimal,
        paper: bool,
    ) -> Result<Trade> {
        if let Some(mut trade) = self.trade_repository.get_trade(trade_id).await? {
            let order_id = if !paper {
                let order = self
                    .clob_client
                    .limit_order()
                    .token_id(U256::from_str(&trade.token_id)?)
                    .size(trade.size)
                    .price(price)
                    .side(Side::Sell)
                    .post_only(true)
                    .build()
                    .await?;

                let signed_order = self.clob_client.sign(&self.signer, order).await?;
                let response = self.clob_client.post_order(signed_order).await?;

                self.wait_for_fill(&response.order_id).await?;

                Some(response.order_id.clone())
            } else {
                None
            };
            trade.close_trade(price, Utc::now().timestamp(), order_id);
            self.trade_repository.update_trade(&trade).await?;

            return Ok(trade);
        } else {
            return Err("Trade not found".into());
        }
    }

    pub async fn cancel_all_orders(&self) -> Result<CancelOrdersResponse> {
        Ok(self.clob_client.cancel_all_orders().await?)
    }
}

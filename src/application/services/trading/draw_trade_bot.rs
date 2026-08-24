use std::{str::FromStr, sync::Arc};

use alloy::primitives::U256;
use chrono::{DateTime, Duration, Utc};
use polymarket_client_sdk_v2::types::dec;
use tokio::{task::JoinHandle, time::sleep};
use tracing::info;

use crate::{
    domain::entities::trade::TradeStrategy,
    infrastructure::{
        config::trade_config::TradeConfig, polymarket_provider::PolymarketProvider,
        repositories::trade_repository::TradeRepository,
    },
    shared::error::Result,
};
use sqlx::SqlitePool;

pub struct DrawTradeBot {
    provider: Arc<PolymarketProvider>,
    handles: Vec<JoinHandle<()>>,
    trade_repository: TradeRepository,
}

impl DrawTradeBot {
    pub async fn new(pool: SqlitePool) -> Result<Self> {
        let draw_trade_bot = DrawTradeBot {
            provider: Arc::new(PolymarketProvider::new(pool.clone()).await?),
            handles: Vec::new(),
            trade_repository: TradeRepository::from_pool(pool),
        };

        draw_trade_bot.trade_repository.run_migrations().await?;

        Ok(draw_trade_bot)
    }

    async fn resume_open_trades(&self) {
        let open_trades = match self.trade_repository.get_open_trades().await {
            Ok(trades) => trades,
            Err(e) => {
                tracing::error!("Error getting open_trades {}", e);
                return;
            }
        };

        for trade in &open_trades {
            let clob_token_id = match U256::from_str(&trade.token_id) {
                Ok(token) => token,
                Err(_) => {
                    tracing::error!("Error parsing token_id from trade to U256");
                    continue;
                }
            };
            let entry_dt = DateTime::from_timestamp(trade.entry_time, 0)
                .expect("invalid entry_time timestamp");
            let sell_time = entry_dt + TradeConfig::BUY_OFFSET + TradeConfig::SELL_OFFSET;
            let exit_delay = sell_time - Utc::now();

            if exit_delay <= Duration::zero() {
                tracing::info!(trade = %trade.id, "Trade past sell time, closing immediately");
            }

            let provider = Arc::clone(&self.provider);
            let trade_id = trade.id.clone();
            let market_id = trade.market_id.clone();
            let paper = trade.paper;

            tokio::spawn(Self::close_trade(
                provider,
                clob_token_id,
                trade_id,
                market_id,
                exit_delay.max(Duration::zero()),
                paper,
            ));
        }
    }

    pub async fn run(&mut self, paper: bool) {
        self.resume_open_trades().await;

        loop {
            match self.run_todays_trades(paper).await {
                Ok(_) => {
                    sleep(std::time::Duration::from_hours(24)).await;
                }
                Err(e) => {
                    tracing::error!(e);
                    sleep(std::time::Duration::from_millis(1000)).await;
                }
            }
        }
    }

    pub async fn run_todays_trades(&mut self, paper: bool) -> Result<()> {
        let todays_draw_markets = self.provider.get_todays_draw_markets().await?;

        for market in todays_draw_markets {
            let Some(start_date) = market.game_start_time else {
                continue;
            };

            let normalized = match start_date.rfind('+') {
                Some(pos) if start_date[pos..].len() == 3 => {
                    format!("{}:00", start_date)
                }
                _ => start_date.clone(),
            };
            let parsed = DateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M:%S%:z")
                .map_err(|e| format!("Invalid date '{}': {e}", start_date))?
                .with_timezone(&Utc);
            let bet_time = parsed - TradeConfig::BUY_OFFSET;
            let delay = bet_time - Utc::now();

            if delay <= Duration::zero() {
                info!(market = %market.id, "Skipping, too late to bet");
                continue;
            }

            let provider = Arc::clone(&self.provider);
            let market_id = market.id.clone();
            let sell_market_id = market.id.clone();

            let clob_token_id = match market.clob_token_ids.as_ref().and_then(|v| v.first()) {
                Some(id) => *id,
                None => {
                    tracing::error!(market = %market.id, "No token id found, skipping");
                    continue;
                }
            };

            let handle: JoinHandle<()> = tokio::spawn(async move {
                let mins = delay.num_minutes();
                let fmt = if mins >= 1440 {
                    format!("{}d {}h", mins / 1440, (mins % 1440) / 60)
                } else if mins >= 60 {
                    format!("{}h {}m", mins / 60, mins % 60)
                } else {
                    format!("{}m", mins)
                };
                info!(market = %market_id, "Scheduled bet in {}", fmt);
                tokio::time::sleep(delay.to_std().unwrap()).await;

                let Ok(current_price) = provider.current_price(clob_token_id).await else {
                    tracing::error!(market = %market_id, "Failed to get current price");
                    return;
                };
                let price = current_price - dec!(0.01);

                if price < TradeConfig::MIN_PRICE || price > TradeConfig::MAX_PRICE {
                    info!("Price out of trade strategy range");
                    return;
                }

                let size = (TradeConfig::BANKROLL * dec!(0.02)) / price;

                match provider
                    .place_trade_and_wait_for_resolution(
                        &market_id,
                        &clob_token_id.to_string(),
                        price,
                        size,
                        paper,
                        TradeStrategy::DrawTimeDecay,
                    )
                    .await
                {
                    Ok(trade) => {
                        let exit_delay = (parsed + TradeConfig::SELL_OFFSET) - Utc::now();
                        let sell_mins = exit_delay.num_minutes();
                        let sell_fmt = if sell_mins >= 60 {
                            format!("{}h {}m", sell_mins / 60, sell_mins % 60)
                        } else {
                            format!("{}m", sell_mins)
                        };
                        info!(trade = %trade.id, "Trade placed, scheduling sell in {}", sell_fmt);

                        let sell_provider = Arc::clone(&provider);
                        let sell_trade_id = trade.id.clone();

                        tokio::spawn(Self::close_trade(
                            sell_provider,
                            clob_token_id,
                            sell_trade_id,
                            sell_market_id,
                            exit_delay,
                            paper,
                        ));
                    }
                    Err(e) => tracing::error!(market = %market_id, "Trade failed: {e}"),
                }
            });

            self.handles.push(handle);
        }

        info!(count = self.handles.len(), "Scheduled bets");
        Ok(())
    }

    async fn close_trade(
        provider: Arc<PolymarketProvider>,
        clob_token_id: U256,
        trade_id: String,
        market_id: String,
        exit_delay: Duration,
        paper: bool,
    ) {
        if exit_delay > Duration::zero() {
            tokio::time::sleep(exit_delay.to_std().unwrap()).await;
        }

        let Ok(current_price) = provider.current_price(clob_token_id).await else {
            tracing::error!(market = %market_id, "Failed to get current price");
            return;
        };
        let sell_price = current_price + dec!(0.01);

        match provider
            .exit_trade_and_wait_for_resolution(&trade_id, sell_price, paper)
            .await
        {
            Ok(t) => info!(trade = %t.id, "Trade closed"),
            Err(e) => tracing::error!(market = %market_id, "Sell failed: {e}"),
        }
    }

    pub async fn cancel_all(&mut self) -> Result<()> {
        self.provider.cancel_all_orders().await?;
        for handle in self.handles.drain(..) {
            handle.abort();
        }
        info!("All scheduled bets cancelled");

        Ok(())
    }
}

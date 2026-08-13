use std::time::Duration;

use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, TimeZone, Utc};
use clap::{Parser, Subcommand};
use num_traits::cast::ToPrimitive;
use polymarket_client_sdk_v2::gamma::Client as GammaClient;
use polymarket_client_sdk_v2::gamma::types::request::EventsRequest;
use reqwest::Client as HttpClient;
use rust_value_betting_engine::infrastructure::repositories::PolymarketRepository;
use serde::Deserialize;
use serde_json;

#[derive(Parser)]
#[command(
    name = "polymarket-cli",
    about = "Fetch and store Polymarket soccer data"
)]
struct Cli {
    #[arg(short = 'd', long = "db-path", default_value = "polymarket_data.db")]
    db_path: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Fetch historical soccer matches from Polymarket Gamma API and store in DB
    FetchMatches {
        /// Only store events with start_date >= this (YYYY-MM-DD)
        #[arg(long)]
        maybe_start_date_min: Option<String>,

        /// Only store events with start_date <= this (YYYY-MM-DD)
        #[arg(long)]
        maybe_start_date_max: Option<String>,

        /// Max events per API request
        #[arg(long, default_value = "100")]
        limit: u32,

        /// Pagination offset
        #[arg(long, default_value = "0")]
        offset: u32,
    },
    /// Fetch OHLCV price history from pmxt.dev for stored markets
    FetchPrices {
        /// Specific market ID (fetches all markets if omitted)
        #[arg(long)]
        market_id: Option<String>,

        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,

        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,

        /// Candle resolution (e.g. 1h, 5m)
        #[arg(long, default_value = "1m")]
        resolution: String,
    },
    /// List events in the database
    List {
        #[arg(long)]
        from: Option<String>,

        #[arg(long)]
        to: Option<String>,

        /// Sort descending (newest first)
        #[arg(long)]
        desc: bool,
    },
    /// Show database statistics
    Info,
    /// Backup the database to db/backups/<datetime>/
    Backup,
    /// Run backtest of draw-value strategy
    Backtest {
        /// Candle resolution in minutes (1 or 5)
        #[arg(long, default_value = "5")]
        resolution_minutes: u32,
    },
    /// Run the draw time decay trading strategy
    DrawTrade {
        /// Run in paper trading mode (no real orders)
        #[arg(long)]
        paper: bool,
    },
}

fn parse_date(s: Option<String>) -> Option<DateTime<Utc>> {
    let s = s.as_ref()?;
    let naive = NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()?;
    naive
        .and_hms_opt(0, 0, 0)
        .map(|dt| Utc.from_utc_datetime(&dt))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();

    let repo = PolymarketRepository::new(&cli.db_path).await?;
    repo.run_migrations().await?;

    match cli.command {
        Command::FetchMatches {
            maybe_start_date_min,
            maybe_start_date_max,
            limit,
            offset,
        } => {
            handle_fetch_matches(
                &repo,
                maybe_start_date_min,
                maybe_start_date_max,
                limit,
                offset,
            )
            .await?;
        }
        Command::FetchPrices {
            market_id,
            from,
            to,
            resolution,
        } => {
            handle_fetch_prices(&repo, market_id, from, to, &resolution).await?;
        }
        Command::List { from, to, desc } => {
            handle_list(&repo, from, to, desc).await?;
        }
        Command::Info => {
            handle_info(&repo).await?;
        }
        Command::Backup => {
            handle_backup(&cli.db_path).await?;
        }
        Command::Backtest { resolution_minutes } => {
            handle_backtest(&repo, resolution_minutes).await?;
        }
        Command::DrawTrade { paper } => {
            handle_execute_draw_trade_strategy(paper).await;
        }
    }

    Ok(())
}

async fn handle_fetch_matches(
    repo: &PolymarketRepository,
    maybe_start_date_min_str: Option<String>,
    maybe_start_date_max_str: Option<String>,
    limit: u32,
    offset: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let gamma = GammaClient::default();
    let maybe_start_date_min = parse_date(maybe_start_date_min_str);
    let maybe_start_date_max = parse_date(maybe_start_date_max_str);

    let request = EventsRequest::builder()
        .closed(true)
        .tag_slug("soccer".to_string())
        .maybe_start_date_min(maybe_start_date_min)
        .maybe_start_date_max(maybe_start_date_max)
        .ascending(true)
        .limit(limit as i32)
        .offset(offset as i32)
        .build();

    let events = gamma.events(&request).await?;

    let fetched_at = chrono::Utc::now().to_rfc3339();
    let mut event_count = 0;
    let mut market_count = 0;

    for event in &events {
        let tags_json = event
            .tags
            .as_ref()
            .map(|t| serde_json::to_string(t).unwrap_or_default());

        let start_date = event.start_date.map(|d| d.to_rfc3339());
        let start_time = event.start_time.map(|d| d.to_rfc3339());
        let end_date = event.end_date.map(|d| d.to_rfc3339());

        repo.insert_event(
            &event.id,
            event.title.as_deref().unwrap_or(""),
            event.slug.as_deref(),
            event.description.as_deref(),
            event.series_slug.as_deref(),
            event.home_team_name.as_deref(),
            event.away_team_name.as_deref(),
            tags_json.as_deref(),
            event.volume.and_then(|v| v.to_f64()),
            event.volume_24hr.and_then(|v| v.to_f64()),
            event.closed.unwrap_or(true),
            start_date.as_deref(),
            start_time.as_deref(),
            end_date.as_deref(),
            &fetched_at,
        )
        .await?;
        event_count += 1;

        if let Some(markets) = &event.markets {
            for market in markets {
                let clob_ids = market.clob_token_ids.as_ref();
                let token_id_yes = clob_ids.and_then(|ids| ids.first().map(|id| id.to_string()));
                let token_id_no = clob_ids.and_then(|ids| ids.get(1).map(|id| id.to_string()));

                let prices = market.outcome_prices.as_ref();
                let price_yes = prices.and_then(|p| p.first().and_then(|v| v.to_f64()));
                let price_no = prices.and_then(|p| p.get(1).and_then(|v| v.to_f64()));

                let market_type = market.market_type.as_deref();
                let status = if market.closed.unwrap_or(false) {
                    Some("closed")
                } else {
                    None
                };

                repo.insert_market(
                    &market.id,
                    &event.id,
                    market.question.as_deref().unwrap_or(""),
                    market.slug.as_deref(),
                    market.description.as_deref(),
                    status,
                    market_type,
                    market.volume.and_then(|v| v.to_f64()),
                    market.volume_24hr.and_then(|v| v.to_f64()),
                    market.liquidity.and_then(|v| v.to_f64()),
                    None,
                    token_id_yes.as_deref(),
                    token_id_no.as_deref(),
                    price_yes,
                    price_no,
                )
                .await?;
                market_count += 1;
            }
        }
    }

    tracing::info!("Fetched {} events, {} markets", event_count, market_count,);
    println!("FETCHED:{event_count}");

    Ok(())
}

async fn handle_fetch_prices(
    repo: &PolymarketRepository,
    market_id: Option<String>,
    from: Option<String>,
    to: Option<String>,
    resolution: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("PMXT_API_KEY").map_err(|_| "PMXT_API_KEY not set in environment or .env")?;

    let client = HttpClient::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let mut markets = if let Some(mid) = &market_id {
        repo.get_markets_for_event(mid).await?
    } else {
        repo.get_markets_without_prices().await?
    };

    // Filter by optional date range (based on event end_date)
    if let Some(from_str) = &from {
        if let Some(from_dt) = parse_date(Some(from_str.clone())) {
            markets.retain(|m| {
                m.end_date
                    .as_deref()
                    .and_then(|d| {
                        NaiveDate::parse_from_str(d.get(..10).unwrap_or(d), "%Y-%m-%d")
                            .ok()
                            .and_then(|nd| nd.and_hms_opt(0, 0, 0))
                            .map(|dt| Utc.from_utc_datetime(&dt))
                    })
                    .map(|dt| dt >= from_dt)
                    .unwrap_or(true)
            });
        }
    }
    if let Some(to_str) = &to {
        if let Some(to_dt) = parse_date(Some(to_str.clone())) {
            markets.retain(|m| {
                m.end_date
                    .as_deref()
                    .and_then(|d| {
                        NaiveDate::parse_from_str(d.get(..10).unwrap_or(d), "%Y-%m-%d")
                            .ok()
                            .and_then(|nd| nd.and_hms_opt(23, 59, 59))
                            .map(|dt| Utc.from_utc_datetime(&dt))
                    })
                    .map(|dt| dt <= to_dt)
                    .unwrap_or(true)
            });
        }
    }

    if markets.is_empty() {
        tracing::info!("No markets to fetch prices for");
        return Ok(());
    }

    let mut total_candles = 0;

    for market in &markets {
        let token_ids: Vec<&str> = market
            .clob_token_id_yes
            .as_deref()
            .into_iter()
            .chain(market.clob_token_id_no.as_deref())
            .collect();

        if token_ids.is_empty() {
            continue;
        }

        // Skip if no match start time to anchor window
        let Some(match_start_str) = &market.match_start else {
            continue;
        };

        // Window: 5h before match start to 2h after (covers pre-match + match + resolution)
        let Some(match_start) = chrono::DateTime::parse_from_rfc3339(match_start_str)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
        else {
            continue;
        };
        let start = match_start - ChronoDuration::hours(5);
        let end = match_start + ChronoDuration::hours(2);
        let from_ts = Some(start.timestamp_millis());
        let to_ts = Some(end.timestamp_millis());

        // Rate-limit: stay well under 60 req/min (observed burst ~10 req)
        tokio::time::sleep(Duration::from_millis(3000)).await;

        for token_id in &token_ids {
            let candles =
                match fetch_ohlcv(&client, &api_key, token_id, resolution, from_ts, to_ts).await {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!(
                            "Failed to fetch candles for market {} token {}: {}",
                            &market.id,
                            token_id,
                            e
                        );
                        continue;
                    }
                };

            for candle in &candles {
                repo.insert_price_candle(
                    &market.id,
                    token_id,
                    candle.timestamp,
                    candle.open,
                    candle.high,
                    candle.low,
                    candle.close,
                    candle.volume,
                )
                .await?;
            }

            total_candles += candles.len();
            tracing::info!(
                "Stored {} candles for market {} token {}",
                candles.len(),
                &market.id,
                token_id,
            );

            tokio::time::sleep(Duration::from_millis(1500)).await;
        }

        repo.set_market_has_prices(&market.id).await?;
    }

    tracing::info!("Done. Stored {} total price candles", total_candles);
    Ok(())
}

async fn fetch_ohlcv(
    client: &HttpClient,
    api_key: &str,
    outcome_id: &str,
    resolution: &str,
    start: Option<i64>,
    end: Option<i64>,
) -> Result<Vec<PriceCandle>, Box<dyn std::error::Error>> {
    let mut url = format!(
        "https://api.pmxt.dev/api/polymarket/fetchOHLCV?outcomeId={}&resolution={}",
        outcome_id, resolution
    );

    if let Some(s) = start {
        url.push_str(&format!("&start={}", s));
    }
    if let Some(e) = end {
        url.push_str(&format!("&end={}", e));
    }

    for attempt in 0..5 {
        let resp = match client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let wait = 10 + attempt * 15;
                tracing::warn!(
                    "Network error (attempt {}/5): {} — retrying in {}s",
                    attempt + 1,
                    e,
                    wait
                );
                tokio::time::sleep(Duration::from_secs(wait)).await;
                continue;
            }
        };

        let status = resp.status();
        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(
                    "Failed to read response body (attempt {}/5): {}",
                    attempt + 1,
                    e
                );
                let wait = 10 + attempt * 15;
                tokio::time::sleep(Duration::from_secs(wait)).await;
                continue;
            }
        };

        if status.is_success() {
            if let Ok(body) = serde_json::from_str::<PmxtResponse>(&text) {
                return Ok(body.data);
            }
            // success but unparseable — maybe empty
            if let Ok(body) = serde_json::from_str::<PmxtErrorResponse>(&text) {
                if !body.success {
                    tracing::warn!("API error: {:?}", body.error);
                    return Ok(Vec::new());
                }
            }
        }

        if text.contains("Rate exceeded") || status == 429 {
            let wait = 10 + attempt * 15;
            tracing::warn!(
                "Rate limited, waiting {}s (attempt {}/5)",
                wait,
                attempt + 1
            );
            tokio::time::sleep(Duration::from_secs(wait)).await;
            continue;
        }

        // Other error — just return empty
        tracing::warn!("OHLCV fetch failed (HTTP {}): {}", status, text);
        return Ok(Vec::new());
    }

    Ok(Vec::new())
}

async fn handle_list(
    repo: &PolymarketRepository,
    from: Option<String>,
    to: Option<String>,
    desc: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let from = from.unwrap_or_else(|| "1970-01-01".to_string());
    let to = to.unwrap_or_else(|| "2099-12-31".to_string());

    let order = if desc { "DESC" } else { "ASC" };
    let events = repo.get_events_in_date_range(&from, &to, order).await?;

    for event in &events {
        let home = event.home_team.as_deref().unwrap_or("?");
        let away = event.away_team.as_deref().unwrap_or("?");
        let date = event
            .start_time
            .as_deref()
            .or(event.start_date.as_deref())
            .unwrap_or("?");
        println!("{} | {} vs {} | {}", event.id, home, away, date);
    }

    println!("Total: {} events", events.len());
    Ok(())
}

async fn handle_info(repo: &PolymarketRepository) -> Result<(), Box<dyn std::error::Error>> {
    let count = repo.event_count().await?;
    println!("Events in DB: {}", count);
    Ok(())
}

async fn handle_backup(db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_dir = format!("db/backups/{}", now);
    std::fs::create_dir_all(&backup_dir)?;

    let db_file = std::path::Path::new(db_path);
    let file_name = db_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("polymarket_data.db");

    for suffix in ["", "-tshm", "-wal"] {
        let src = format!("{}{}", db_path, suffix);
        if std::path::Path::new(&src).exists() {
            let dst = format!("{}/{}{}", backup_dir, file_name, suffix);
            std::fs::copy(&src, &dst)?;
        }
    }

    println!("Backup saved to {}/", backup_dir);
    Ok(())
}

async fn handle_backtest(
    repo: &PolymarketRepository,
    resolution_minutes: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    use rust_value_betting_engine::application::backtesting::backtest_runner::{
        BacktestConfig, BacktestRunner,
    };

    let config = BacktestConfig {
        resolution_minutes,
        from: None,
        to: None,
    };

    let metrics = BacktestRunner::run(repo, &config).await?;

    println!("Backtest: draw-value | resolution={}m", resolution_minutes);
    println!("Markets: {}", metrics.total_markets);
    println!("Markets with data: {}", metrics.markets_with_data);
    println!("Trades: {}", metrics.total_trades);
    println!("Win rate: {:.1}%", metrics.win_rate * 100.0);
    println!("Total P&L: {:.4}", metrics.total_pnl);
    println!("Avg P&L: {:.4}", metrics.avg_pnl);
    println!("Max drawdown: {:.4}", metrics.max_drawdown);
    println!("Sharpe: {:.2}", metrics.sharpe_ratio);
    Ok(())
}

async fn handle_execute_draw_trade_strategy(paper: bool) {
    use rust_value_betting_engine::application::services::trading::draw_trade_bot::DrawTradeBot;

    let mut draw_trade_bot = match DrawTradeBot::default().await {
        Ok(bot) => bot,
        Err(e) => {
            tracing::error!("Failed to initialize trade bot: {e}");
            return;
        }
    };

    draw_trade_bot.run(paper).await;
}

#[derive(Debug, Deserialize)]
struct PmxtResponse {
    #[allow(dead_code)]
    success: bool,
    data: Vec<PriceCandle>,
}

#[derive(Debug, Deserialize)]
struct PmxtErrorResponse {
    #[allow(dead_code)]
    success: bool,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct PriceCandle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

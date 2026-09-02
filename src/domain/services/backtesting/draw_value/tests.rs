use chrono::DateTime;

use super::*;
use crate::infrastructure::repositories::MarketRow;

const MINUTE_MS: i64 = 60_000;

fn market(match_start: Option<&str>) -> MarketRow {
    MarketRow {
        id: "market-1".to_string(),
        event_id: "event-1".to_string(),
        question: Some("Will the match end in a draw?".to_string()),
        clob_token_id_yes: Some("yes-token".to_string()),
        clob_token_id_no: Some("no-token".to_string()),
        match_start: match_start.map(|s| s.to_string()),
        end_date: None,
    }
}

fn candle(minutes_before_kickoff: i64, close: f64, start_ms: i64) -> PriceCandle {
    PriceCandle {
        timestamp: start_ms + minutes_before_kickoff * MINUTE_MS,
        open: close,
        high: close,
        low: close,
        close,
        volume: 100.0,
    }
}

#[test]
fn strategy_name_is_draw_value() {
    assert_eq!("draw-value", DrawValueStrategy::new(1).name());
}

#[test]
fn evaluate_returns_no_signals_without_match_start() {
    let strategy = DrawValueStrategy::new(1);
    let candles = vec![candle(-5, 0.10, 0)];

    assert!(strategy.evaluate(&market(None), &candles).is_empty());
}

#[test]
fn evaluate_returns_no_signals_for_unparseable_match_start() {
    let strategy = DrawValueStrategy::new(1);
    let candles = vec![candle(-5, 0.10, 0)];

    assert!(
        strategy
            .evaluate(&market(Some("not-a-date")), &candles)
            .is_empty()
    );
}

#[test]
fn evaluate_returns_no_signals_when_exit_candle_is_missing() {
    let strategy = DrawValueStrategy::new(1);
    let start_ms = DateTime::parse_from_rfc3339("2026-05-10T18:00:00Z")
        .unwrap()
        .timestamp_millis();
    let candles = vec![candle(-5, 0.10, start_ms)];

    assert!(
        strategy
            .evaluate(&market(Some("2026-05-10T18:00:00Z")), &candles)
            .is_empty()
    );
}

#[test]
fn evaluate_enters_inside_buy_window_and_exits_ten_minutes_after_kickoff() {
    let strategy = DrawValueStrategy::new(1);
    let start_ms = DateTime::parse_from_rfc3339("2026-05-10T18:00:00Z")
        .unwrap()
        .timestamp_millis();
    let exit_close = 0.05;
    let candles = vec![
        candle(-30, 0.10, start_ms),
        candle(-5, 0.10, start_ms),
        candle(20, exit_close, start_ms),
        candle(40, 0.02, start_ms),
    ];

    let signals = strategy.evaluate(&market(Some("2026-05-10T18:00:00Z")), &candles);

    assert_eq!(1, signals.len());
    let signal = &signals[0];
    assert_eq!("market-1", signal.market_id);
    assert_eq!(start_ms - 5 * MINUTE_MS, signal.entry_time);
    // strategy buys one cent below the close
    assert!((0.09 - signal.entry_price).abs() < 1e-9);
    assert_eq!(start_ms + 10 * MINUTE_MS, signal.exit_time);
    assert!((exit_close - signal.exit_price).abs() < 1e-9);
}

#[test]
fn evaluate_skips_candles_outside_the_price_band() {
    let strategy = DrawValueStrategy::new(1);
    let start_ms = DateTime::parse_from_rfc3339("2026-05-10T18:00:00Z")
        .unwrap()
        .timestamp_millis();
    // close 0.50 -> adjusted price 0.49 is above MAX_PRICE (0.18)
    let candles = vec![
        candle(-5, 0.50, start_ms),
        candle(20, 0.05, start_ms),
    ];

    let signals = strategy.evaluate(&market(Some("2026-05-10T18:00:00Z")), &candles);

    assert!(signals.is_empty());
}

#[test]
fn evaluate_ignores_candles_at_or_after_kickoff_for_entry() {
    let strategy = DrawValueStrategy::new(1);
    let start_ms = DateTime::parse_from_rfc3339("2026-05-10T18:00:00Z")
        .unwrap()
        .timestamp_millis();
    // only cheap candles exist after kickoff: entry must never happen there
    let candles = vec![candle(0, 0.10, start_ms), candle(20, 0.05, start_ms)];

    let signals = strategy.evaluate(&market(Some("2026-05-10T18:00:00Z")), &candles);

    assert!(signals.is_empty());
}

#[test]
fn evaluate_ignores_candles_older_than_the_buy_window() {
    let strategy = DrawValueStrategy::new(1);
    let start_ms = DateTime::parse_from_rfc3339("2026-05-10T18:00:00Z")
        .unwrap()
        .timestamp_millis();
    // candle 11 minutes before kickoff is outside the 10 minute window
    let candles = vec![candle(-11, 0.10, start_ms), candle(20, 0.05, start_ms)];

    let signals = strategy.evaluate(&market(Some("2026-05-10T18:00:00Z")), &candles);

    assert!(signals.is_empty());
}

#[test]
fn evaluate_returns_at_most_one_signal_per_market() {
    let strategy = DrawValueStrategy::new(1);
    let start_ms = DateTime::parse_from_rfc3339("2026-05-10T18:00:00Z")
        .unwrap()
        .timestamp_millis();
    let candles = vec![
        candle(-9, 0.10, start_ms),
        candle(-8, 0.10, start_ms),
        candle(20, 0.05, start_ms),
    ];

    let signals = strategy.evaluate(&market(Some("2026-05-10T18:00:00Z")), &candles);

    assert_eq!(1, signals.len());
}

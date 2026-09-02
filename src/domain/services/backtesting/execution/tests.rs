use super::*;
use crate::infrastructure::repositories::MarketRow;

struct FixedStrategy {
    signals: Vec<Signal>,
}

impl Strategy for FixedStrategy {
    fn name(&self) -> &str {
        "fixed"
    }

    fn evaluate(&self, _market: &MarketRow, _candles: &[PriceCandle]) -> Vec<Signal> {
        self.signals.clone()
    }
}

struct CountingStrategy;

impl Strategy for CountingStrategy {
    fn name(&self) -> &str {
        "counting"
    }

    fn evaluate(&self, _market: &MarketRow, candles: &[PriceCandle]) -> Vec<Signal> {
        vec![Signal {
            market_id: "m".to_string(),
            entry_time: 0,
            entry_price: 0.0,
            exit_time: 1,
            exit_price: candles.first().map(|c| c.close).unwrap_or(0.0),
        }]
    }
}

fn row(id: &str) -> MarketRow {
    MarketRow {
        id: id.to_string(),
        event_id: "e".to_string(),
        question: None,
        clob_token_id_yes: None,
        clob_token_id_no: None,
        match_start: None,
        end_date: None,
    }
}

fn signal(entry: f64, exit: f64) -> Signal {
    Signal {
        market_id: "m".to_string(),
        entry_time: 0,
        entry_price: entry,
        exit_time: 1,
        exit_price: exit,
    }
}

fn candles(closes: &[f64]) -> Vec<PriceCandle> {
    closes
        .iter()
        .enumerate()
        .map(|(i, &close)| PriceCandle {
            timestamp: i as i64,
            open: close,
            high: close,
            low: close,
            close,
            volume: 0.0,
        })
        .collect()
}

#[test]
fn simulator_reports_empty_metrics_without_markets() {
    let metrics = TradeSimulator::run(&FixedStrategy { signals: vec![] }, &[], |_| vec![]);

    assert_eq!(0, metrics.total_markets);
    assert_eq!(0, metrics.total_trades);
    assert_eq!(0.0, metrics.win_rate);
    assert_eq!(0.0, metrics.total_pnl);
}

#[test]
fn simulator_skips_markets_without_candles() {
    let markets = vec![row("with-data"), row("without-data")];
    let metrics = TradeSimulator::run(
        &CountingStrategy,
        &markets,
        |id| if id == "with-data" { candles(&[0.1]) } else { vec![] },
    );

    assert_eq!(2, metrics.total_markets);
    assert_eq!(1, metrics.markets_with_data);
    assert_eq!(1, metrics.total_trades);
}

#[test]
fn simulator_counts_wins_losses_and_pnl() {
    let strategy = FixedStrategy {
        signals: vec![signal(0.10, 0.15), signal(0.10, 0.05), signal(0.20, 0.25)],
    };
    let metrics = TradeSimulator::run(&strategy, &[row("m1")], |_| candles(&[0.1]));

    assert_eq!(1, metrics.total_markets);
    assert_eq!(3, metrics.total_trades);
    assert_eq!(2, metrics.winning_trades);
    assert_eq!(1, metrics.losing_trades);
    assert!((metrics.win_rate - 2.0 / 3.0).abs() < 1e-9);
    assert!((metrics.total_pnl - (0.05 - 0.05 + 0.05)).abs() < 1e-9);
    assert!((metrics.avg_pnl - 0.05 / 3.0).abs() < 1e-9);
}

#[test]
fn simulator_tracks_max_drawdown_of_cumulative_pnl() {
    // returns: +0.1, -0.2, +0.05 -> equity 0.1, -0.1, -0.05
    // drawdown reaches 0.2 when equity drops from peak 0.1 to -0.1
    let strategy = FixedStrategy {
        signals: vec![signal(0.0, 0.1), signal(0.0, -0.2), signal(0.0, 0.05)],
    };
    let metrics = TradeSimulator::run(&strategy, &[row("m1")], |_| candles(&[0.1]));

    assert!((metrics.max_drawdown - 0.2).abs() < 1e-9);
}

#[test]
fn simulator_zeroes_sharpe_for_single_trade() {
    let strategy = FixedStrategy { signals: vec![signal(0.1, 0.2)] };
    let metrics = TradeSimulator::run(&strategy, &[row("m1")], |_| candles(&[0.1]));

    assert_eq!(1, metrics.total_trades);
    assert_eq!(0.0, metrics.sharpe_ratio);
}

#[test]
fn simulator_computes_sharpe_for_consistent_returns() {
    // returns identical and exactly representable -> std dev 0 -> sharpe 0
    let strategy = FixedStrategy {
        signals: vec![signal(0.0, 0.5), signal(0.0, 0.5), signal(0.0, 0.5)],
    };
    let metrics = TradeSimulator::run(&strategy, &[row("m1")], |_| candles(&[0.5]));
    assert_eq!(0.0, metrics.sharpe_ratio);

    // mixed returns -> positive sharpe when mean > 0
    let strategy = FixedStrategy {
        signals: vec![signal(0.0, 0.2), signal(0.0, 0.1), signal(0.0, 0.15)],
    };
    let metrics = TradeSimulator::run(&strategy, &[row("m1")], |_| candles(&[0.1]));
    assert!(metrics.sharpe_ratio > 0.0);
}

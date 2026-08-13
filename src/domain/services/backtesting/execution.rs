use crate::domain::services::backtesting::{BacktestMetrics, PriceCandle, Signal, Strategy};
use crate::infrastructure::repositories::MarketRow;

pub struct TradeSimulator;

impl TradeSimulator {
    pub fn run(
        strategy: &dyn Strategy,
        markets: &[MarketRow],
        mut candles_lookup: impl FnMut(&str) -> Vec<PriceCandle>,
    ) -> BacktestMetrics {
        let mut all_signals: Vec<Signal> = Vec::new();
        let mut markets_with_data = 0usize;

        for market in markets {
            let candles = candles_lookup(&market.id);
            if candles.is_empty() {
                continue;
            }
            markets_with_data += 1;
            let signals = strategy.evaluate(market, &candles);
            all_signals.extend(signals);
        }

        let total_trades = all_signals.len();
        let winning_trades = all_signals.iter().filter(|s| s.exit_price > s.entry_price).count();
        let losing_trades = all_signals.iter().filter(|s| s.exit_price <= s.entry_price).count();
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        let total_pnl: f64 = all_signals
            .iter()
            .map(|s| s.exit_price - s.entry_price)
            .sum();

        let avg_pnl = if total_trades > 0 {
            total_pnl / total_trades as f64
        } else {
            0.0
        };

        let mut running = 0.0;
        let mut peak = 0.0;
        let mut max_drawdown = 0.0;
        let mut returns: Vec<f64> = Vec::with_capacity(all_signals.len());

        for signal in &all_signals {
            let r = signal.exit_price - signal.entry_price;
            returns.push(r);
            running += r;
            if running > peak {
                peak = running;
            }
            let dd = peak - running;
            if dd > max_drawdown {
                max_drawdown = dd;
            }
        }

        let sharpe_ratio = if returns.len() > 1 {
            let mean = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns
                .iter()
                .map(|r| (r - mean).powi(2))
                .sum::<f64>()
                / (returns.len() as f64 - 1.0);
            let std_dev = variance.sqrt();
            if std_dev > 0.0 {
                mean / std_dev * (total_trades as f64).sqrt()
            } else {
                0.0
            }
        } else {
            0.0
        };

        BacktestMetrics {
            total_markets: markets.len(),
            markets_with_data,
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            total_pnl,
            avg_pnl,
            max_drawdown,
            sharpe_ratio,
        }
    }
}

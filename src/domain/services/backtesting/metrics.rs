#[derive(Debug, Default)]
pub struct BacktestMetrics {
    pub total_markets: usize,
    pub markets_with_data: usize,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
}

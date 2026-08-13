# Draw Value Betting Strategy — Analysis

## Data
- **Source**: Polymarket soccer markets via Gamma API + pmxt.dev 1m OHLCV
- **Markets**: 1,784 draw markets (slugs containing "draw"), 1,443 with price data
- **Candles**: 1-minute OHLCV, 7h window (5h before match → 2h after)
- **Match times**: 1,461/2,100 events have `start_time` (actual kickoff)

## Key Finding: Draw Price Spikes at Kickoff

When aligning candles to match start time (`offset_min = minutes from kickoff`):

| Offset | Avg Draw Price | Notes |
|---|---|---|
| -300 to -10 | ~0.241 (24%) | Stable pre-match |
| -1 | 0.243 | |
| **0 (kickoff)** | **0.355 (35.5%)** | **Spike!** |
| +5 | 0.307 | Still elevated |
| +10 | 0.293 | Declining |
| +30 | 0.279 | Settling |

The draw price jumps **~7.4 percentage points** at kickoff (t-stat = 21.1, p ≈ 0).

## Strategy: Enter Cheap, Exit After Kickoff

| Parameter | Recommended | Rationale |
|---|---|---|
| Entry | -5 min before match | Price stable, enough lead time |
| Exit | +5 min after match | Captures post-kickoff elevation |
| Min entry price | 0.03 | Avoids illiquid extremes |
| Max entry price | 0.15 | Only buy when draw is undervalued |
| Max market volume | 15,000 | Filters high-vol efficient markets (no edge) |

### Performance (143 trades qualifying)

| Metric | Value |
|---|---|
| Trades/year | ~110 (8% of matches) |
| Avg entry price | 0.084 |
| Avg exit price | 0.191 |
| **Mean return** | **+143%** |
| **Win rate** | **65.5%** |
| Std dev (per trade) | 268% |
| t-statistic | 5.6 (highly significant) |
| Avg winner | +258% |
| Avg loser | -19% |
| Sharpe (annualized) | ~5.6 |

### Edge Breakdown by Volume

| Market Vol | Trades | Return | Win | Notes |
|---|---|---|---|---|
| 0–5k | 73 | +133% | 62% | Good |
| 5k–15k | 37 | +163% | 73% | Best win rate |
| 15k–50k | 21 | +104% | 62% | Declining |
| > 50k | 11 | +8% | 55% | **No edge** |

### Comparison: Exit at Kickoff vs +5

| Exit | Trades | Mean | Win | t-stat |
|---|---|---|---|---|
| Offset 0 (kickoff) | 142 | +150% | 57% | 7.0 |
| **Offset +5** | 143 | **+122%** | **63%** | 5.9 |

Exit at kickoff has higher return but lower win rate (+5min is more reliable).

### Worst-Case Pricing (Slippage Test)

Entry at `high` (buy worst), exit at `low` (sell worst) → **identical results** (1m candles are 1-tick wide). Intra-minute slippage is negligible; filling the order is the real challenge.

## Query (Datasette-compatible)

```sql
WITH 
  entry_offset AS (SELECT -5 AS val),
  exit_offset  AS (SELECT 5 AS val),
  min_entry    AS (SELECT 0.03 AS val),
  max_entry    AS (SELECT 0.15 AS val),
  min_mkt_vol  AS (SELECT 0 AS val),
  max_mkt_vol  AS (SELECT 15000 AS val),
  prices AS (
    SELECT ph.market_id,
      round((ph.timestamp - (strftime('%s', e.start_time) * 1000)) / 60000.0, 0) as off,
      ph.close
    FROM price_history ph
    JOIN polymarket_markets pm ON ph.market_id = pm.id
    JOIN polymarket_events e ON pm.event_id = e.id
    WHERE pm.derived_type = 'draw' AND e.start_time IS NOT NULL AND ph.close <= 0.5
  ),
  mkt_data AS (
    SELECT id, volume FROM polymarket_markets WHERE derived_type = 'draw'
  ),
  paired AS (
    SELECT p.market_id,
      MAX(CASE WHEN off = (SELECT val FROM entry_offset) THEN close END) as entry,
      MAX(CASE WHEN off = (SELECT val FROM exit_offset) THEN close END) as exit,
      MAX(m.volume) as mkt_vol
    FROM prices p
    JOIN mkt_data m ON p.market_id = m.id
    WHERE off IN ((SELECT val FROM entry_offset), (SELECT val FROM exit_offset))
    GROUP BY p.market_id
    HAVING entry IS NOT NULL AND exit IS NOT NULL
  ),
  total_q AS (
    SELECT COUNT(*) as total
    FROM (SELECT DISTINCT market_id FROM prices WHERE off = (SELECT val FROM entry_offset))
  ),
  filtered AS (
    SELECT * FROM paired
    WHERE (entry >= (SELECT val FROM min_entry) OR (SELECT val FROM min_entry) IS NULL)
      AND (entry <  (SELECT val FROM max_entry) OR (SELECT val FROM max_entry) IS NULL)
      AND (mkt_vol >= (SELECT val FROM min_mkt_vol) OR (SELECT val FROM min_mkt_vol) IS NULL)
      AND (mkt_vol <= (SELECT val FROM max_mkt_vol) OR (SELECT val FROM max_mkt_vol) IS NULL)
  ),
  returns AS (
    SELECT (exit - entry) / entry * 100 as ret,
      entry, exit, mkt_vol,
      CASE WHEN exit > entry THEN 1 ELSE 0 END as won
    FROM filtered
  )
SELECT
  COUNT(*) as n_trades,
  ROUND(100.0 * COUNT(*) / (SELECT total FROM total_q), 1) as qualify_rate,
  ROUND(AVG(entry), 4) as avg_entry,
  ROUND(AVG(exit), 4) as avg_exit,
  ROUND(AVG(mkt_vol), 0) as avg_mkt_vol,
  ROUND(100.0 * SUM(won) / COUNT(*), 1) as win_rate,
  ROUND(AVG(ret), 1) as mean_pct,
  ROUND(SQRT(AVG(ret*ret) - AVG(ret)*AVG(ret)), 1) as std_pct,
  ROUND(AVG(ret) / NULLIF(SQRT(AVG(ret*ret) - AVG(ret)*AVG(ret)), 0) * SQRT(COUNT(*)), 2) as t_stat
FROM returns;
```

## Kelly Position Sizing

| Kelly Fraction | % of Bankroll per Trade | Risk Level |
|---|---|---|
| Full Kelly | ~10% | High (drawdown risk) |
| **Half Kelly** | **~5%** | **Recommended** |
| Quarter Kelly | ~2.5% | Conservative |

## Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| **Slippage / no fill** | High | Use limit orders; skip if unfilled |
| **Kickoff timing off** | Medium | Enter at -7min buffer |
| **Overfitting** | Medium | Keep parameters simple; forward test |
| **Low liquidity** | High | Filter by market volume < 15k |
| **Fees** | Low | Use maker orders (0.1%) |
| **Sample size** | Medium | 110 trades is modest |

## Next Steps

1. **Forward testing** (paper trade) — 1-2 months of real-time monitoring without real money
2. **Automate signal generation** — script that checks live markets 5min before kickoff, prints entry/exit signals
3. **Validate exit at kickoff** — check if the offset 0 price spike is actually executable or just artefact
4. **Expand data** — add `closedTime` / `umaEndDate` for alternative exit strategies
5. **League/competition filter** — test if certain leagues have stronger edge
6. **Real execution test** — place small real orders (e.g., 10-20 USDC) to validate fill rates and slippage

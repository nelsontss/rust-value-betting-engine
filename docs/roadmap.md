1. [X] Project foundation

   1.1 [X] Initialize Rust crate and workspace structure

   1.2 [X] Add core domain, application, infrastructure, and shared modules

   1.3 [X] Add baseline smoke test and Cargo configuration

2. [X] Cross-platform fixture clustering

   2.1 [X] Model games with normalized team, competition, country, and date data

   2.2 [X] Implement fuzzy fixture matching and similarity scoring

   2.3 [X] Build `ClusterService` to group equivalent fixtures across bookmakers

   2.4 [X] Add clustering regression tests for fuzzy matching scenarios

3. [X] Market domain modeling

   3.1 [X] Introduce `Market`, `MarketType`, `Line`, and `Odd` domain types

   3.2 [X] Model match result, moneyline, total, handicap, and asian handicap markets

   3.3 [X] Enforce positive odds through `Odd::new` validation

   3.4 [X] Improve line canonicalization for `MarketType` grouping keys

   3.5 [X] Derive `Game` market map keys from `MarketType::from(&market)` during construction and updates

   3.6 [X] Encapsulate `Game` markets behind a read-only getter plus controlled update methods

   3.7 [X] Add `Game` tests for market indexing, replacement, and logical-type expansion

4. [X] Fixture cluster market aggregation

   4.1 [X] Introduce `FixtureCluster` as a domain entity

   4.2 [X] Encapsulate fixture membership and grouped market indexing inside `FixtureCluster`

   4.3 [X] Add tests for grouped markets across the same fixture on different platforms

   4.4 [X] Support persistent grouped market lookup while keeping game state as the source of truth

5. [X] Arbitrage calculation engine

   5.1 [X] Add dedicated arbitrage domain models in `arbitrage.rs`

   5.2 [X] Implement arbitrage detection for match result and moneyline markets

   5.3 [X] Implement line-aware arbitrage detection for totals, handicaps, and asian handicaps

   5.4 [X] Handle push and quarter-line scenarios in total and asian handicap calculations

   5.5 [X] Add generic arbitrage metrics such as stake distribution, payout, profit, and ROI

   5.6 [X] Add arbitrage-focused unit tests for markets and arbitrage entities

6. [X] Shared mutable game state architecture

   6.1 [X] Move fixture clustering to the entity layer

   6.2 [X] Replace borrowed game references with `SharedGame`

   6.3 [X] Adopt `Arc<RwLock<Game>>` for shared mutable game state

   6.4 [X] Update cluster and market tests to the shared-game architecture

   6.5 [X] Update `FixtureCluster` and `ClusterService` callers to consume the new `Game` market API

7. [X] Owned game state and incremental cluster updates

   7.1 [X] Replace `SharedGame` usage with owned `Game` values in `FixtureCluster` and `ClusterService`

   7.2 [X] Rework fixture-cluster market indexing to track unique game IDs per `MarketType`

   7.3 [X] Support `FixtureCluster::update_markets` reindexing when a clustered game gains new market types

   7.4 [X] Propagate arbitrage results when `ClusterService::update_games` falls back to `add_games`

   7.5 [X] Add `FixtureCluster` regression tests for duplicate IDs, unknown updates, reindexing, and order-insensitive grouped markets

    7.6 [X] Add `ClusterService` regression tests for `update_games` on known games, unknown matching games, and unknown distinct fixtures

8. [X] Date-partitioned cluster storage

    8.1 [X] Refactor `ClusterService::clusters` from flat `HashMap<String, FixtureCluster>` to `HashMap<NaiveDateTime, HashMap<String, FixtureCluster>>` for date-scoped lookup

    8.2 [X] Add `cluster_id_to_date` reverse-lookup map to support `update_markets` and `insert_games` across the nested structure

    8.3 [X] Add `ClusterService::update_markets` for batch market updates via game IDs

    8.4 [X] Fix `and_modify` no-op bug on vacant entries by switching to `or_insert_with(HashMap::new)` in the cluster creation path

    8.5 [X] Fix test `assert_cluster_sizes` to flatten the nested HashMap when computing cluster counts and sizes

 9. [X] Performance benchmarks

     9.1 [X] Add Criterion.rs dev-dependency and benchmark harness configuration

     9.2 [X] Remove synthetic-data benchmark generators; replace with platform pipeline benchmarks

     9.3 [X] Implement parser throughput benchmarks for `BetanoParser` and `LeBullParser` at 10/100/1000 event counts

     9.4 [X] Implement engine ingestion benchmarks: `insert_games` for distinct and clustering scenarios at 10/100/1000 game counts

     9.5 [X] Implement market update benchmarks simulating live odds updates at 10/100/500 simultaneous updates

     9.6 [X] Implement cross-platform clustering benchmark testing 2/5/10 platforms on the same fixture

     9.7 [X] Implement cluster query latency benchmarks for `get_clusters` (10–2000 clusters) and `get_cluster_by_id` (O(1) lookup)

     9.8 [X] Implement serialization throughput benchmark for `ClusterResponse` JSON output

     9.9 [X] Implement SSE broadcast latency benchmark measuring tokio broadcast channel delivery

      9.10 [ ] Establish baseline performance targets and regression alerting

      9.11 [ ] Validate benchmarks against realistic production-like data shapes

10. [X] Chrome extension data ingestion

    10.1 [X] Create Chrome extension with native messaging support

    10.2 [X] Implement Betano API polling service in background service worker

    10.3 [X] Add Unix socket bridge for extension-to-engine communication

    10.4 [X] Implement length-prefixed message framing over stdin/stdout

    10.5 [X] Add upcoming games API polling alongside today's games

    10.6 [X] Log file rotation with configurable line cap in bridge binary

11. [X] Domain type improvements

    11.1 [X] Move `Platform` enum to domain entities layer

    11.2 [X] Replace `String` platform field on `Game` with typed `Platform` enum

    11.3 [X] Add serde support for `Platform` serialization/deserialization

12. [X] Bridge infrastructure

    12.1 [X] Define `BridgeMessage` tagged enum for socket communication

    12.2 [X] Implement `BridgeConnector` as Unix stream client

    12.3 [X] Implement `BetanoParser` for converting Betano API JSON to domain models

    12.4 [X] Wire `BookmakerScrapperService` with connector and cluster service

13. [X] BetanoParser typeId corrections

    13.1 [X] Remove incorrect typeId=9 → AsianHandicap mapping (typeId=9 is Double Chance)

    13.2 [X] Map typeId=10 (Draw No Bet) to Moneyline

    13.3 [X] Map typeId=14 (Over/Under 1st Half) to Total market

     13.4 [X] Skip typeId=15 (Both Teams to Score) — not a moneyline equivalent

14. [X] Connector resilience and extensibility

     14.1 [ ] Auto-reconnect loop for bridge socket disconnections

     14.2 [X] Plugin-style `DataParser` registry for multi-platform support without modifying `BridgeConnector`

15. [X] Double Chance market type

       15.1 [X] Add `DoubleChanceMarket` struct with 3 selections (1X, 12, X2)
       15.2 [X] Add `DoubleChance` variant to `Market` enum and `MarketGroup`
       15.3 [X] Implement `arbitrage_opportunites` for double chance markets
       15.4 [X] Re-enable typeId=9 parsing in `BetanoParser` mapped to `Market::DoubleChance`

16. [X] LeBull platform integration

      16.1 [X] Map LeBull upcoming API response structure (stakeType mapping)

      16.2 [X] Create `LeBullParser` implementing `DataParser` trait

      16.3 [X] Create `LeBullConnector` with HTTP polling loop via `ureq`

      16.4 [X] Wire `LeBullConnector` into `BookmakerScrapperService::run`

      16.5 [ ] Register `LeBullConnector` reconnect/retry logic

      16.6 [ ] Add `DoubleChance` market variant (blocked on 15) and map stakeType 37

       16.7 [X] Update `UPCOMING_URL` with `leagueTimeFilter=14` and full stakeType list from website
       16.8 [ ] Evaluate stakeType 80, 356, 545, 702, 724, 144, 176415, 183254, 217797, 313638, 313639, 357318 (unmapped types in full request)

17. [X] Async runtime migration

      17.1 [X] Add tokio dependency with full features

      17.2 [X] Swap `std::sync::mpsc` for `tokio::sync::mpsc` in connectors and service

      17.3 [X] Make `BookmakerScrapperService::run` async

      17.4 [X] Add tokio `broadcast` channel for live cluster update notifications

      17.5 [X] Add `ClusterService::subscribe_to_game_updates` for downstream consumers

      17.6 [X] Migrate `main` to `#[tokio::main]` with `tokio::select!` for graceful shutdown

18. [X] SSE web server

      18.1 [X] Add `axum`, `tower-http`, `tokio-stream`, `tracing`, `tracing-subscriber` dependencies

      18.2 [X] Create `infrastructure::server` module with SSE endpoint for cluster events

      18.3 [X] Serve cluster list and individual cluster detail via JSON endpoints

      18.4 [X] Wire server startup alongside the engine in `main`

19. [X] Tracing and logging infrastructure

      19.1 [X] Add `tracing` and `tracing-subscriber` with env-filter support

      19.2 [X] Remove raw `println!` from connectors and parsers

      19.3 [X] Initialize tracing subscriber with `RUST_LOG` env-var support in `main`

20. [X] FixtureCluster enhancements

      20.1 [X] Add `updated_at` timestamp tracking on cluster mutations

      20.2 [X] Add `representative_game` field for Betano-first display preference

      20.3 [X] Add `games()` accessor with deterministic sort (Betano first, then by ID)

      20.4 [X] Expose `MatchResultMarket` fields as `pub` for external access

21. [X] Parser test coverage

      21.1 [X] Add tests for `BetanoParser`: empty input, all typeId branches, selection validation, multiple events/blocks

      21.2 [X] Add tests for `LeBullParser`: all stakeType branches, date parsing, live filtering, multiple leagues/lines

      21.3 [X] Add tests for `ParserRegistry`: registration, dispatch, unknown platform

       21.4 [X] Add tests for `Platform` enum: variants, serde round-trip

22. [X] Frontend routing and games pages

       22.1 [X] Add `@tanstack/react-router` with code-based route tree
       22.2 [X] Create root layout with sticky navigation bar (Clusters / Games links)
       22.3 [X] Add `/games` route rendering all games in a grid
       22.4 [X] Add `/games/$platform` route for platform-filtered games view
       22.5 [X] Create `useGames` and `usePlatformGames` hooks with TanStack Query
       22.6 [X] Add platform badge links for navigation between all games and platform views

23. [X] Polymarket live connector

       23.1 [X] Add `Platform::Polymarket` and game/market mapping from Polymarket events (moneyline, match result/int\/5, double chance, totals, spreads)
       23.2 [X] Fetch upcoming soccer events from Gamma REST API (48h window, paginated) in `polymarket_connector.rs`
       23.3 [X] Stream live prices over the market WebSocket channel with `custom_feature_enabled` and 10s heartbeat (raw `tokio-tungstenite`)
       23.4 [X] Discover newly listed markets via `new_market` events and subscribe incrementally to their price channels
       23.5 [X] Feed live Polymarket games into `ClusterService` for diff extraction, with 1h snapshot poll for reconciliation

24. [X] Cluster outcome diff statistics

       24.1 [X] Compute `diff` per market outcome: `Polymarket implied prob − median bookmaker implied prob` in `cluster_service.rs`
       24.2 [X] Add `QuantileMultiset` order-statistics structure backing percentile computation
       24.3 [X] Aggregate per `(MarketType, Outcome)` into `ClusterStatistics` with mean, median, p05, p25, p75, p95 and sample count
       24.4 [X] Broadcaster `StatisticsUpdated` from `ClusterService` and SSE endpoint `GET /statistics` in the server
       24.5 [X] Add statistics DTOs and tests for percentiles and aggregation

25. [X] Trade domain and persistence

       25.1 [X] Add `Trade` entity with `TradeStatus` lifecycle (open/closed/cancelled/expired), `TradeStrategy`, side, stake, entry/exit price, PnL, and paper flag
       25.2 [X] Add `TradeRepository` on SQLite with `trades` table migration and insert/get/update/open-trades queries
       25.3 [X] Register `trade` module and repository in the module trees

26. [X] Polymarket execution provider

       26.1 [X] Build authenticated CLOB client (`localSignerType` from `POLYMARKET_PRIVATE_KEY`) in `polymarket_provider.rs`
       26.2 [X] Query current price for a token id and fetch today's soccer draw markets from local DB
       26.3 [X] Place signed post-only limit orders with fill polling and `wait_for_trade` resolution
       26.4 [X] Exit open trades with take-profit/price-offset logic and cancel-all support
       26.5 [X] Paper mode recording trades without placing real orders

27. [X] Polymarket data pipeline and CLI

       27.1 [X] Add `PolymarketRepository` on SQLite: `polymarket_events`, `polymarket_markets`, and OHLCV price history tables
       27.2 [X] Add `polymarket-cli` binary: `fetch-matches` (Gamma history backfill), `fetch-prices` (pmxt.dev candles, needs `PMXT_API_KEY`)
       27.3 [X] Add `list`, `info`, and `backup` DB inspection commands
       27.4 [X] Add `TradeConfig` (bankroll, max/min price bands, buy/sell offsets) and wire into the trading module

28. [X] Draw-value strategy trading and backtesting

       28.1 [X] Implement `DrawValueStrategy` signal (buy low-priced draw markets) and `TradeSimulator` execution engine
       28.2 [X] Compute `BacktestMetrics`: win rate, total/avg PnL, max drawdown, Sharpe ratio
       28.3 [X] Add `BacktestRunner` over stored OHLCV candles with a `backtest` CLI command
       28.4 [X] Add `DrawTimeDecay` trade bot: today's draw markets with volume filter, scheduled buy ~10min before kickoff, auto-sell, resume of open trades on restart
       28.5 [X] Expose `draw-trade` CLI command with `--paper` mode

29. [X] Bwin platform integration

        29.1 [X] Add `Platform::Bwin` variant with serde round-trip tests
        29.2 [X] Create `BwinParser` for SignalR `FixtureUpdate` payloads with test coverage
        29.3 [X] Create `BwinConnector`: one-shot fixtures fetch + SignalR WebSocket subscription per fixture topic
        29.4 [X] Wire `BwinConnector` into `BookmakerScrapperService::run`
        29.5 [X] Expose Bwin via `/platforms` route

30. [X] Market history persistence

        30.1 [X] Add `MarketDataPoint` entity for timestamped market snapshots
        30.2 [X] Add `GameRepository` on SQLite with `games`/market-point tables and migrations
        30.3 [X] Replace `market_history_service` with `MarketService` broadcasting new market updates over a broadcast channel
        30.4 [X] Serve per-game market history via `GET /market-history` backed by `GameRepository`

31. [X] Cluster and statistics persistence refactor

        31.1 [X] Add `FixtureClusterRepository` on SQLite persisting clusters and computed diffs
        31.2 [X] Extract `StatisticsService` from cluster statistics aggregation
        31.3 [X] Rebuild historical diff distributions from persisted diffs at startup, then update incrementally via `DashMap`
        31.4 [X] Update SSE statistics endpoint to consume `StatisticsService`

32. [X] Frontend statistics dashboard

        32.1 [X] Add `/statistics` route and `StatisticsPage` component
        32.2 [X] Add `useClusterStatistics` hook and statistics types
        32.3 [X] Refactor clusters view into `ClusterTable` + `ClusterInspector`
        32.4 [X] Capture UI spec in `web-app/section-dashboard-snapshot.md`

33. [X] Value alerts (see `docs/alerts.md`)

        33.1 [X] Add `AlertService` / `AlertEvent::MarketClusterDiffDivergency` subscribing to `ClusterService` cluster updates (`src/domain/services/alert_service.rs`)
        33.2 [X] Compare per-outcome live diff against `StatisticsService` p05/p95 thresholds (outside-band → alert) with broadcast channel
        33.3 [X] Add `alert_response` DTO and SSE endpoint `GET /alerts` (`src/infrastructure/server/dto/alert_response.rs`, `routes/alerts.rs`)
        33.4 [X] Wire `AlertService` in `main.rs` and expose via `AppState` / `routes.rs`
        33.5 [X] Add frontend alerts store, `useAlerts` hook, `AlertsPage`/`AlertsToaster`, `/alerts` route and nav entry (`web-app/src/stores/alerts.ts`, `hooks/useAlerts.ts`, `components/alerts/*`)
        33.6 [X] Document design in `docs/alerts.md`

34. [X] Live (in-play) data from all platforms (see `docs/live-data-plan.md`)

        34.1 [X] No `Phase`/`is_live` on `Game` — live coverage via same `InsertGames`/`UpdateMarkets` flow
        34.2 [X] Betano: discover live API `GET /danae-webapi/api/live/overview/latest` + extension `pollLive` normalization to `{ blocks }` shape
        34.3 [X] LeBull: add `LIVE_URL` (`/leagues/inplay`) polling and remove `isLive` skip in `lebull_parser.rs`
        34.4 [X] Bwin: handle `MainToLiveUpdate`/`FixtureUpdate`/`OptionMarketUpdate`/`Close` frames, accept `Suspended` markets, chunked subscribe (40 topics/msg), exponential-backoff reconnect loop
        34.5 [X] Statistics/diffs treat all snapshots uniformly

35. [X] Game deep-link and diff model

        35.1 [X] Add `link: Option<Url>` to `Game` (`game.rs` + `game/tests.rs` + parsers set link) and expose via `game_response` DTO
        35.2 [X] Refactor `FixtureCluster` diff storage from `HashMap<(MarketType, Outcome), _>` to `HashMap<MarketType, HashMap<Outcome, _>>` (`fixture_cluster.rs`, `cluster_service.rs`, repositories)
        35.3 [X] Refactor `StatisticsService` `historical_stats` → `historical_diffs: DashMap<MarketType, HashMap<Outcome, ClusterStatistics>>` and nested `get_historical_statistics`
        35.4 [X] Persist nested diff shape in `FixtureClusterRepository` / `GameRepository` migrations and update DTOs (`cluster_response`, `statistics_response`)
        35.5 [X] Add `regex` crate and `url` usage for link handling

36. [X] Connector and parser improvements

        36.1 [X] Polymarket: switch Gamma pagination to `events/keyset` (`limit=500` + `after_cursor`), add `GammaKeysetResponse`, send `UpdateMarkets` for existing games
        36.2 [X] Bwin: chunk topics into 40-per-message subscribes, handle `Close` frame, reconnect resilience
        36.3 [X] Betano/LeBull/Bwin parsers: live-market handling (`Suspended` acceptance in `bwin_parser`, live inclusion in `lebull_parser`, `betano_parser` updates)
        36.4 [X] Benchmarks: update `Game::new` calls with `None` link, rename `subscribe_to_game_updates` → `subscribe_to_cluster_updates`

37. [X] Frontend improvements

        37.1 [X] Add `LiveDiffComparisonTable` and wire into `ClusterInspector` alongside `MarketChart`/`MarketHistory`
        37.2 [X] Platform deep-links: `ClusterInspector`/`GameCard` show `ExternalLink` to `game.link`, `ClusterTable`/`Dashboard`/`MarketGroupTable` updates
        37.3 [X] Statistics refactor: extract `stores/statistics.ts`, simplify `useClusterStatistics`, update `StatisticsPage` and `lib/markets`
        37.4 [X] Routing/nav: add `Alerts` to `__root.tsx` / `routeTree.ts` and `cluster` type `link` field

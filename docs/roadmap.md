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

9. [ ] Performance benchmarks

    9.1 [X] Add Criterion.rs dev-dependency and benchmark harness configuration

    9.2 [X] Implement benchmark data generators (`src/benchmark/data.rs`) for distinct fixtures, clusters, arbitrage scenarios, and market-variant games

    9.3 [X] Implement throughput benchmarks for `add_games` and `insert_games` across varying counts and platform densities

    9.4 [X] Implement latency benchmarks for arbitrage detection under no load, steady load, bursts, stale updates, and capacity curves

    9.5 [X] Implement CPU/memory benchmarks for per-game insert cost and service initialization throughput

    9.6 [X] Implement response-time benchmarks for similarity scoring, cluster arbitrage scans, market-group arbitration, and fixture matching

     9.7 [ ] Establish baseline performance targets and regression alerting

     9.8 [ ] Validate benchmarks against realistic production-like data shapes

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

    13.4 [X] Map typeId=15 (Both Teams to Score) to Moneyline

14. [X] Connector resilience and extensibility

    14.1 [ ] Auto-reconnect loop for bridge socket disconnections

    14.2 [X] Plugin-style `DataParser` registry for multi-platform support without modifying `BridgeConnector`

15. [ ] Double Chance market type

    15.1 [ ] Add `DoubleChanceMarket` struct with 3 selections (1X, 12, X2)

    15.2 [ ] Add `DoubleChance` variant to `Market` enum and `MarketGroup`

    15.3 [ ] Implement `arbitrage_opportunites` for double chance markets

    15.4 [ ] Re-enable typeId=9 parsing in `BetanoParser` mapped to `Market::DoubleChance`

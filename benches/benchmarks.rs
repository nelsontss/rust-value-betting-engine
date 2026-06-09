use std::{hint::black_box, sync::Arc};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use serde_json::json;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

use rust_value_betting_engine::domain::{
    ClusterService, Game,
    entities::{Market, Platform},
};
use rust_value_betting_engine::infrastructure::{
    parsers::{betano_parser::BetanoParser, lebull_parser::LeBullParser},
    server::dto::cluster_response::ClusterResponse,
};

// ---------------------------------------------------------------------------
// Helpers: data generation
// ---------------------------------------------------------------------------

const TEAMS: &[&str] = &[
    "FC Porto",
    "SL Benfica",
    "Sporting CP",
    "SC Braga",
    "Vitoria SC",
    "Manchester United",
    "Manchester City",
    "Liverpool",
    "Arsenal",
    "Chelsea",
    "FC Barcelona",
    "Real Madrid",
    "Atletico Madrid",
    "Juventus",
    "Inter Milan",
    "AC Milan",
    "Bayern Munich",
    "Borussia Dortmund",
    "Paris Saint-Germain",
    "Ajax",
];

const COUNTRIES: &[&str] = &[
    "Portugal",
    "England",
    "Spain",
    "Italy",
    "Germany",
    "France",
    "Netherlands",
];

const COMPETITIONS: &[&str] = &[
    "Liga Portugal",
    "Premier League",
    "La Liga",
    "Serie A",
    "Bundesliga",
    "Ligue 1",
    "Eredivisie",
];

const PLATFORMS: &[Platform] = &[Platform::Betano, Platform::LeBull];

fn fixture_date(day_offset: u32, hour: u32, min: u32) -> NaiveDateTime {
    let base = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let date = base + chrono::Duration::days(day_offset as i64);
    NaiveDateTime::new(
        date,
        NaiveTime::from_hms_milli_opt(hour.min(23), min, 0, 0).unwrap(),
    )
}

fn generate_games(count: usize, with_markets: bool, distinct: bool) -> Vec<Game> {
    (0..count)
        .map(|i| {
            let day = i as u32 / 50;
            let hour = 10 + (i % 12) as u32;
            let home_idx = i % TEAMS.len();
            let away_idx = if distinct {
                (i + 1 + i / TEAMS.len()) % TEAMS.len()
            } else {
                (i + 1) % TEAMS.len()
            };
            let country_idx = i % COUNTRIES.len();
            let competition_idx = i % COMPETITIONS.len();

            let markets = if with_markets {
                vec![
                    Market::total(
                        &format!("t-{}", i),
                        2.5,
                        1.85 + (i as f64 * 0.001).rem_euclid(0.3),
                        1.95 - (i as f64 * 0.001).rem_euclid(0.2),
                    )
                    .unwrap(),
                    Market::moneyline(
                        &format!("ml-{}", i),
                        1.8 + (i as f64 * 0.001).rem_euclid(0.5),
                        2.0 + (i as f64 * 0.001).rem_euclid(0.4),
                    )
                    .unwrap(),
                ]
            } else {
                vec![]
            };

            Game::new(
                TEAMS[home_idx],
                TEAMS[away_idx],
                COUNTRIES[country_idx],
                COMPETITIONS[competition_idx],
                fixture_date(day, hour, 30),
                PLATFORMS[i % PLATFORMS.len()],
                markets,
            )
        })
        .collect()
}

/// Load a ClusterService with a certain number of games.
fn load_service(
    game_count: usize,
    with_markets: bool,
    distinct: bool,
) -> Arc<RwLock<ClusterService>> {
    let games = generate_games(game_count, with_markets, distinct);
    let mut service = ClusterService::new();
    service.insert_games(games);
    Arc::new(RwLock::new(service))
}

// ---------------------------------------------------------------------------
// Benchmark: Parser throughput — BetanoParser
// ---------------------------------------------------------------------------

fn betano_json_event(id: &str, home: &str, away: &str, type_ids: &[i64]) -> serde_json::Value {
    let markets: Vec<serde_json::Value> = type_ids
        .iter()
        .map(|&tid| {
            let selections = match tid {
                1 => vec![
                    json!({"price": 2.0}),
                    json!({"price": 3.2}),
                    json!({"price": 4.0}),
                ],
                10 | 13 | 14 => vec![json!({"price": 1.8}), json!({"price": 2.1})],
                _ => vec![json!({"price": 2.0})],
            };
            json!({
                "typeId": tid,
                "id": format!("mkt-{}-{}", id, tid),
                "handicap": 0.0,
                "selections": selections,
            })
        })
        .collect();

    json!({
        "id": id,
        "name": format!("{} - {}", home, away),
        "leagueName": "Liga Portugal",
        "regionName": "Portugal",
        "startTime": 1_777_000_000_000i64,
        "markets": markets,
    })
}

fn make_betano_payload(event_count: usize) -> serde_json::Value {
    let events: Vec<serde_json::Value> = (0..event_count)
        .map(|i| {
            betano_json_event(
                &format!("evt-{}", i),
                TEAMS[i % TEAMS.len()],
                TEAMS[(i + 1) % TEAMS.len()],
                &[1, 10, 13],
            )
        })
        .collect();

    json!({"blocks": [{"events": events}]})
}

fn bench_betano_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser/betano");
    group.sampling_mode(SamplingMode::Auto);

    for event_count in [10, 100, 1000] {
        let payload = make_betano_payload(event_count);
        group.throughput(Throughput::Elements(event_count as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(event_count),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let games = BetanoParser::parse_data(black_box(payload.clone()));
                    black_box(games.len())
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Parser throughput — LeBullParser
// ---------------------------------------------------------------------------

fn make_lebull_payload(event_count: usize) -> serde_json::Value {
    let games: Vec<serde_json::Value> = (0..event_count)
        .map(|i| {
            json!({
                "eventId": i as i64,
                "teamA": TEAMS[i % TEAMS.len()],
                "teamB": TEAMS[(i + 1) % TEAMS.len()],
                "isLive": false,
                "date": format!("/Date({})/", 1_777_000_000_000i64),
                "stakeTypes": [
                    {"stakeTypeId": 1, "stakes": [
                        {"stakeCode": 1, "stakeArgument": 0.0, "betFactor": 2.0},
                        {"stakeCode": 2, "stakeArgument": 0.0, "betFactor": 3.2},
                        {"stakeCode": 3, "stakeArgument": 0.0, "betFactor": 4.0},
                    ]},
                    {"stakeTypeId": 3, "stakes": [
                        {"stakeCode": 1, "stakeArgument": 2.5, "betFactor": 1.9},
                        {"stakeCode": 2, "stakeArgument": 2.5, "betFactor": 1.9},
                    ]},
                ],
            })
        })
        .collect();

    json!([{
        "countryName": "Portugal",
        "leagueName": "Liga Portugal",
        "games": games,
    }])
}

fn bench_lebull_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser/lebull");
    group.sampling_mode(SamplingMode::Auto);

    for event_count in [10, 100, 1000] {
        let payload = make_lebull_payload(event_count);
        group.throughput(Throughput::Elements(event_count as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(event_count),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let games = LeBullParser::parse_data(black_box(payload.clone()));
                    black_box(games.len())
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Engine ingestion throughput
// ---------------------------------------------------------------------------

fn bench_engine_insert_games(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let mut group = c.benchmark_group("engine/insert_games");
    group.sampling_mode(SamplingMode::Auto);

    for game_count in [10, 100, 1000] {
        let games = generate_games(game_count, true, true);
        group.throughput(Throughput::Elements(game_count as u64));
        group.bench_with_input(
            criterion::BenchmarkId::new("distinct", game_count),
            &games,
            |b, games| {
                b.to_async(&rt).iter(|| async {
                    let service = Arc::new(RwLock::new(ClusterService::new()));
                    let mut svc = service.write().await;
                    svc.insert_games(black_box(games.clone()));
                    black_box(svc.get_clusters().len());
                });
            },
        );
    }

    for game_count in [10, 100, 1000] {
        let games = generate_games(game_count, true, false);
        group.throughput(Throughput::Elements(game_count as u64));
        group.bench_with_input(
            criterion::BenchmarkId::new("clustering", game_count),
            &games,
            |b, games| {
                b.to_async(&rt).iter(|| async {
                    let service = Arc::new(RwLock::new(ClusterService::new()));
                    let mut svc = service.write().await;
                    svc.insert_games(black_box(games.clone()));
                    black_box(svc.get_clusters().len());
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Engine ingestion with market updates (simulating live updates)
// ---------------------------------------------------------------------------

fn bench_engine_market_updates(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let mut group = c.benchmark_group("engine/market_updates");
    group.sampling_mode(SamplingMode::Auto);

    for game_count in [10, 100, 500] {
        let initial_games = generate_games(game_count, false, false);
        let mut update_games = generate_games(game_count, true, false);
        // Give them the same IDs as the initial games so they act as updates
        for (i, game) in update_games.iter_mut().enumerate() {
            if i < initial_games.len() {
                game.id = initial_games[i].id.clone();
            }
        }

        group.throughput(Throughput::Elements(game_count as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(game_count),
            &(initial_games, update_games),
            |b, (initial, updates)| {
                b.to_async(&rt).iter(|| async {
                    let service = Arc::new(RwLock::new(ClusterService::new()));
                    {
                        let mut svc = service.write().await;
                        svc.insert_games(black_box(initial.clone()));
                    }
                    let mut svc = service.write().await;
                    svc.insert_games(black_box(updates.clone()));
                    black_box(svc.get_clusters().len());
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Cluster query latency
// ---------------------------------------------------------------------------

fn bench_get_clusters(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let mut group = c.benchmark_group("query/get_clusters");
    group.sampling_mode(SamplingMode::Auto);

    for cluster_count in [10, 100, 500, 2000] {
        let service = load_service(cluster_count, true, true);
        group.throughput(Throughput::Elements(cluster_count as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(cluster_count),
            &service,
            |b, service| {
                b.to_async(&rt).iter(|| async {
                    let svc = service.read().await;
                    let clusters = svc.get_clusters();
                    black_box(clusters.len());
                });
            },
        );
    }
    group.finish();
}

fn bench_get_cluster_by_id(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let mut group = c.benchmark_group("query/get_cluster_by_id");
    group.sampling_mode(SamplingMode::Auto);

    for cluster_count in [10, 100, 500] {
        let service = load_service(cluster_count, true, true);
        // Grab a known cluster ID
        let cluster_id = {
            let svc = service.blocking_read();
            let clusters = svc.get_clusters();
            clusters.first().map(|c| c.key()).unwrap_or_default()
        };

        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(cluster_count),
            &(service, cluster_id),
            |b, (service, id)| {
                let id = id.clone();
                b.to_async(&rt).iter(|| async {
                    let svc = service.read().await;
                    let result = svc.get_cluster(&id);
                    black_box(result.is_ok());
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Serialization throughput
// ---------------------------------------------------------------------------

fn bench_cluster_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization/cluster_response");
    group.sampling_mode(SamplingMode::Auto);

    for cluster_count in [1, 10, 100] {
        let service = load_service(cluster_count, true, true);
        let clusters: Vec<ClusterResponse> = {
            let svc = service.blocking_read();
            svc.get_clusters()
                .iter()
                .map(|c| ClusterResponse::from(c))
                .collect()
        };

        group.throughput(Throughput::Elements(cluster_count as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(cluster_count),
            &clusters,
            |b, clusters| {
                b.iter(|| {
                    let json = serde_json::to_string(black_box(clusters)).unwrap();
                    black_box(json.len());
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Cluster query under concurrent read load
// ---------------------------------------------------------------------------

fn bench_get_clusters_concurrent(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let mut group = c.benchmark_group("query/get_clusters_concurrent");
    group.sampling_mode(SamplingMode::Auto);

    for cluster_count in [100, 500, 2000] {
        let service = load_service(cluster_count, true, true);

        for &readers in &[1, 5, 10, 25] {
            group.throughput(Throughput::Elements((cluster_count * readers) as u64));
            group.bench_with_input(
                criterion::BenchmarkId::new(
                    format!("r{}c{}", readers, cluster_count),
                    cluster_count,
                ),
                &(service.clone(), readers),
                |b, (svc, r_count)| {
                    let r_count = *r_count;
                    b.to_async(&rt).iter(|| async {
                        let mut handles = Vec::with_capacity(r_count);
                        for _ in 0..r_count {
                            let svc = Arc::clone(svc);
                            handles.push(tokio::spawn(async move {
                                let guard = svc.read().await;
                                let _ = black_box(guard.get_clusters().len());
                            }));
                        }
                        for h in handles {
                            h.await.unwrap();
                        }
                    });
                },
            );
        }
    }
    group.finish();
}

fn bench_get_cluster_by_id_concurrent(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let mut group = c.benchmark_group("query/get_cluster_by_id_concurrent");
    group.sampling_mode(SamplingMode::Auto);

    for cluster_count in [100, 500, 2000] {
        let service = load_service(cluster_count, true, true);
        let cluster_id = {
            let svc = service.blocking_read();
            svc.get_clusters()
                .first()
                .map(|c| c.key())
                .unwrap_or_default()
        };

        for &readers in &[1, 5, 10, 25] {
            group.throughput(Throughput::Elements(readers as u64));
            group.bench_with_input(
                criterion::BenchmarkId::new(
                    format!("r{}c{}", readers, cluster_count),
                    cluster_count,
                ),
                &(service.clone(), cluster_id.clone(), readers),
                |b, (svc, id, r_count)| {
                    let r_count = *r_count;
                    let id = id.clone();
                    b.to_async(&rt).iter(|| async {
                        let mut handles = Vec::with_capacity(r_count);
                        for _ in 0..r_count {
                            let svc = Arc::clone(svc);
                            let id = id.clone();
                            handles.push(tokio::spawn(async move {
                                let guard = svc.read().await;
                                let _ = black_box(guard.get_cluster(&id).is_ok());
                            }));
                        }
                        for h in handles {
                            h.await.unwrap();
                        }
                    });
                },
            );
        }
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: SSE broadcast latency
// ---------------------------------------------------------------------------

fn bench_sse_broadcast(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let mut group = c.benchmark_group("sse/broadcast");
    group.sampling_mode(SamplingMode::Auto);

    // With 20 teams and distinct=false, the first 20 games all have
    // unique team pairs (0 broadcasts). Games 21+ repeat pairs and
    // trigger broadcasts. Use multiples of 20 for consistent baselines.
    for games_per_update in [20, 40, 100] {
        let games = generate_games(games_per_update, false, false);
        group.throughput(Throughput::Elements(games_per_update as u64));

        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(games_per_update),
            &games,
            |b, games| {
                b.to_async(&rt).iter(|| async {
                    let mut service = ClusterService::new();
                    let mut rx = service.subscribe_to_game_updates();
                    // Insert games — this triggers broadcast for clusters with >1 game
                    service.insert_games(black_box(games.clone()));
                    let mut count = 0usize;
                    while let Ok(cluster) = rx.try_recv() {
                        black_box(&cluster);
                        count += 1;
                    }
                    black_box(count);
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: SSE concurrent connections capacity
// ---------------------------------------------------------------------------

fn bench_sse_concurrent_connections(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let mut group = c.benchmark_group("sse/concurrent_connections");
    group.sampling_mode(SamplingMode::Auto);

    // All games share the same fixture so every insertion after the first
    // triggers a broadcast (cluster.game_count() > 1).
    let make_games = |count: usize| -> Vec<Game> {
        (0..count)
            .map(|i| {
                Game::new(
                    "FC Porto",
                    "SL Benfica",
                    "Portugal",
                    "Liga Portugal",
                    fixture_date(0, 15, 30),
                    PLATFORMS[i % PLATFORMS.len()],
                    vec![],
                )
            })
            .collect()
    };

    for &subscribers in &[100, 1000, 10_000] {
        for &games_to_insert in &[0, 10, 100] {
            let games = make_games(games_to_insert);
            // First game creates the cluster (no broadcast); subsequent
            // games each trigger one broadcast to all subscribers.
            let broadcasts = games_to_insert.saturating_sub(1);
            let deliveries = subscribers * broadcasts;

            group.throughput(Throughput::Elements(deliveries.max(1) as u64));
            group.bench_with_input(
                criterion::BenchmarkId::new(
                    format!("{}sub_{}game", subscribers, games_to_insert),
                    subscribers,
                ),
                &(games, subscribers),
                |b, (games, subs)| {
                    let subs = *subs;
                    let games = games.clone();
                    b.to_async(&rt).iter(|| {
                        let games = games.clone();
                        async move {
                            let mut service = ClusterService::new();

                            // 1. Register all subscribers (simulating SSE connections)
                            let mut receivers = Vec::with_capacity(subs);
                            for _ in 0..subs {
                                receivers.push(service.subscribe_to_game_updates());
                            }

                            // 2. Insert games — triggers broadcasts per cluster update
                            service.insert_games(black_box(games));

                            // 3. Drain every subscriber's buffer
                            let mut total = 0usize;
                            for rx in &mut receivers {
                                loop {
                                    match rx.try_recv() {
                                        Ok(cluster) => {
                                            black_box(&cluster);
                                            total += 1;
                                        }
                                        Err(
                                            tokio::sync::broadcast::error::TryRecvError::Empty,
                                        ) => break,
                                        Err(_) => break,
                                    }
                                }
                            }
                            black_box(total);
                        }
                    });
                },
            );
        }
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Cross-platform clustering (the core domain operation)
// ---------------------------------------------------------------------------

fn bench_cross_platform_clustering(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let mut group = c.benchmark_group("engine/cross_platform");
    group.sampling_mode(SamplingMode::Auto);

    for platforms in [2, 5, 10] {
        let games: Vec<Game> = (0..platforms)
            .map(|i| {
                Game::new(
                    "FC Porto",
                    "SL Benfica",
                    "Portugal",
                    "Liga Portugal",
                    fixture_date(0, 15, 30),
                    PLATFORMS[i % PLATFORMS.len()],
                    vec![
                        Market::total(
                            &format!("t-{}", i),
                            2.5,
                            1.85 + (i as f64 * 0.02),
                            1.95 - (i as f64 * 0.01),
                        )
                        .unwrap(),
                    ],
                )
            })
            .collect();

        group.throughput(Throughput::Elements(platforms as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(platforms),
            &games,
            |b, games| {
                b.to_async(&rt).iter(|| async {
                    let service = Arc::new(RwLock::new(ClusterService::new()));
                    let mut svc = service.write().await;
                    svc.insert_games(black_box(games.clone()));
                    let clusters = svc.get_clusters();
                    let arbitrages: Vec<_> = clusters
                        .iter()
                        .flat_map(|c| c.arbitrage_opportunites())
                        .collect();
                    black_box(arbitrages.len());
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion configuration
// ---------------------------------------------------------------------------

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .warm_up_time(std::time::Duration::from_millis(300))
        .measurement_time(std::time::Duration::from_secs(3));
    targets =
        bench_betano_parser,
        bench_lebull_parser,
        bench_engine_insert_games,
        bench_engine_market_updates,
        bench_get_clusters,
        bench_get_cluster_by_id,
        bench_get_clusters_concurrent,
        bench_get_cluster_by_id_concurrent,
        bench_cluster_serialization,
        bench_sse_broadcast,
        bench_sse_concurrent_connections,
        bench_cross_platform_clustering,
);

criterion_main!(benches);

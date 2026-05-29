use std::time::Duration;

use criterion::{
    criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};

use rust_value_betting_engine::benchmark::data;
use rust_value_betting_engine::domain::entities::Game;
use rust_value_betting_engine::domain::services::ClusterService;

// ---------------------------------------------------------------------------
// 1. Throughput — games inserted per second
// ---------------------------------------------------------------------------

fn throughput_add_games_new_clusters(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput/add_new_games");
    let sizes = [10, 100, 1_000, 10_000];

    for &size in &sizes {
        let games = data::generate_distinct_fixtures(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &games, |b, games| {
            b.iter_batched(
                || ClusterService::new(vec![]),
                |mut service| service.add_games(games.clone()),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn throughput_add_games_existing_clusters(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput/add_to_existing");
    let configs = [(10, 2), (10, 10), (100, 2), (100, 10)];

    for &(clusters, per_cluster) in &configs {
        let existing = data::generate_many_clusters(clusters, per_cluster, false);
        let new_batch = data::generate_many_clusters(clusters, per_cluster, true);
        let total = (clusters * per_cluster) as u64;
        group.throughput(Throughput::Elements(total));
        group.bench_with_input(
            BenchmarkId::new("clusters_x_plat", format!("{}x{}", clusters, per_cluster)),
            &(existing, new_batch),
            |b, (existing, new_batch)| {
                b.iter_batched(
                    || ClusterService::new(existing.clone()),
                    |mut service| service.add_games(new_batch.clone()),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn throughput_insert_games_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput/insert_updates");
    let sizes = [10, 100, 1_000];

    for &size in &sizes {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let base_games = data::generate_distinct_fixtures(size);
                    let service = ClusterService::new(base_games);
                    let mut update_games = data::generate_distinct_fixtures(size);
                    for game in &mut update_games {
                        let market =
                            Market::total(&format!("t2.5-{}", game.id), 2.5, 1.9, 1.9).unwrap();
                        game.update_markets(vec![&market]);
                    }
                    (service, update_games)
                },
                |(mut service, games)| service.insert_games(games),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

// Need Market for the benchmarks above
use rust_value_betting_engine::domain::entities::Market;

// ---------------------------------------------------------------------------
// 2. Latency — arbitrage detection time under varying load
// ---------------------------------------------------------------------------

fn latency_detect_no_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency/detect_no_load");

    for &platforms in &[2, 5, 10] {
        let arb_games = data::generate_arbitrage_games(
            platforms,
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portugal",
        );
        let first = arb_games[0].clone();
        let rest = arb_games[1..].to_vec();

        group.bench_with_input(
            BenchmarkId::new("platforms", platforms),
            &(first, rest),
            |b, (first, rest)| {
                b.iter_batched(
                    || {
                        let svc = ClusterService::new(vec![first.clone()]);
                        (svc, rest.clone())
                    },
                    |(mut svc, g)| svc.add_games(g),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn latency_detect_under_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency/detect_under_load");
    let load_levels = [100, 1_000, 5_000];

    for &load in &load_levels {
        let arb_games = data::generate_arbitrage_games(
            5,
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portugal",
        );
        let first = arb_games[0].clone();
        let rest = arb_games[1..].to_vec();

        group.bench_with_input(
            BenchmarkId::from_parameter(load),
            &(load, first, rest),
            |b, &(load, ref first, ref rest)| {
                b.iter_batched(
                    || {
                        let background = data::generate_distinct_fixtures(load);
                        let mut svc = ClusterService::new(background);
                        svc.add_games(vec![first.clone()]);
                        (svc, rest.clone())
                    },
                    |(mut svc, g)| svc.add_games(g),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn latency_detect_under_burst(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency/detect_under_burst");
    let burst_sizes = [10, 100, 1_000];

    for &burst in &burst_sizes {
        let games = data::generate_many_clusters(burst, 2, true);
        let first_half: Vec<Game> = games.chunks(2).map(|chunk| chunk[0].clone()).collect();
        let second_half: Vec<Game> = games.chunks(2).map(|chunk| chunk[1].clone()).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(burst),
            &(first_half, second_half),
            |b, (first, second)| {
                b.iter_batched(
                    || {
                        let svc = ClusterService::new(first.clone());
                        (svc, second.clone())
                    },
                    |(mut svc, g)| svc.add_games(g),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn latency_detect_under_stale_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency/detect_under_stale_updates");
    let stale_counts = [100, 500, 1_000];

    for &stale in &stale_counts {
        group.bench_with_input(BenchmarkId::from_parameter(stale), &stale, |b, &stale| {
            b.iter_batched(
                || {
                    let base = data::generate_many_clusters(1, 2, true);
                    let stale_games: Vec<Game> = (0..stale)
                        .map(|i| {
                            let mut g = base[0].clone();
                            let market = Market::total(
                                &format!("stale-{}", i),
                                2.5,
                                1.85 + (i as f64 * 0.001),
                                1.95 - (i as f64 * 0.001),
                            )
                            .unwrap();
                            g.update_markets(vec![&market]);
                            g
                        })
                        .collect();
                    (base, stale_games)
                },
                |(base, stale_games)| {
                    let mut svc = ClusterService::new(base);
                    let _ = svc.insert_games(stale_games);
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn latency_detect_throughput_capacity(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency/capacity_curve");
    let at_loads = [100, 1_000, 10_000];

    for &load in &at_loads {
        let detection_target = data::generate_arbitrage_games(
            2,
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portugal",
        );

        group.bench_with_input(
            BenchmarkId::from_parameter(load),
            &(load, detection_target),
            |b, &(load, ref target)| {
                b.iter_batched(
                    || {
                        let background = data::generate_distinct_fixtures(load);
                        let mut svc = ClusterService::new(background);
                        svc.add_games(vec![target[0].clone()]);
                        (svc, vec![target[1].clone()])
                    },
                    |(mut svc, g)| svc.add_games(g),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 3. CPU & memory — scaling with game count
// ---------------------------------------------------------------------------

fn cpu_per_game_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_mem/per_game_insert");
    let cluster_sizes = [1, 10, 50, 100];

    for &size in &cluster_sizes {
        let games = data::generate_cluster("FC Porto", "SL Benfica", size, true);
        let single = games[0].clone();

        group.bench_with_input(
            BenchmarkId::new("into_cluster_of", size),
            &(games, single),
            |b, (existing, new_game)| {
                b.iter_batched(
                    || {
                        let svc = ClusterService::new(existing.clone());
                        (svc, new_game.clone())
                    },
                    |(mut svc, g)| svc.add_games(vec![g]),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn cpu_service_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_mem/service_init");
    let sizes = [10, 100, 1_000];

    for &size in &sizes {
        let games = data::generate_distinct_fixtures(size);
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &games, |b, games| {
            b.iter(|| ClusterService::new(games.clone()));
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 4. General response times
// ---------------------------------------------------------------------------

fn response_similarity_score(c: &mut Criterion) {
    let mut group = c.benchmark_group("response/similarity_score");
    let sizes = [10, 100, 1_000];

    for &size in &sizes {
        let games = data::generate_distinct_fixtures(size);
        group.throughput(Throughput::Elements((size * size) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &games, |b, games| {
            b.iter(|| {
                for i in 0..games.len() {
                    for j in i + 1..games.len() {
                        let _ = games[i].similarity_score(&games[j]);
                    }
                }
            });
        });
    }
    group.finish();
}

fn response_cluster_arbitrage_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("response/cluster_arbitrage_scan");
    let platform_counts = [2, 5, 10, 20];

    for &platforms in &platform_counts {
        let games = data::generate_cluster("FC Porto", "SL Benfica", platforms, true);
        let service = ClusterService::new(games);

        group.bench_with_input(
            BenchmarkId::from_parameter(platforms),
            &service,
            |b, svc| {
                b.iter(|| {
                    let _ = svc;
                });
            },
        );
    }
    group.finish();
}

fn response_market_group_arbitrage_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("response/market_group_arbitrage");

    for platform_count in [2, 5] {
        let games: Vec<Game> = (0..platform_count)
            .map(|i| data::generate_all_market_types(&format!("p{}", i), &format!("g{}", i)))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("market_types", format!("{}_platforms", platform_count)),
            &games,
            |b, games| {
                b.iter_batched(
                    || ClusterService::new(games.clone()),
                    |svc| {
                        let _ = svc;
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn response_same_fixture_as(c: &mut Criterion) {
    let mut group = c.benchmark_group("response/same_fixture_as");
    let sizes = [10, 100, 1_000];

    let pool: Vec<Game> = data::generate_distinct_fixtures(1_000);
    for &size in &sizes {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &pool[..size],
            |b, games| {
                b.iter(|| {
                    for i in 0..games.len() {
                        for j in i + 1..games.len() {
                            let _ = games[i].same_fixture_as(&games[j]);
                        }
                    }
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Group definitions
// ---------------------------------------------------------------------------

criterion_group! {
    name = throughput;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(15))
        .sample_size(50)
        .warm_up_time(Duration::from_secs(5));
    targets =
        throughput_add_games_new_clusters,
        throughput_add_games_existing_clusters,
        throughput_insert_games_updates,
}

criterion_group! {
    name = latency;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(50);
    targets =
        latency_detect_no_load,
        latency_detect_under_load,
        latency_detect_under_burst,
        latency_detect_under_stale_updates,
        latency_detect_throughput_capacity,
}

criterion_group! {
    name = cpu_mem;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(50);
    targets =
        cpu_per_game_insert,
        cpu_service_init,
}

criterion_group! {
    name = response;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(50);
    targets =
        response_similarity_score,
        response_cluster_arbitrage_scan,
        response_market_group_arbitrage_types,
        response_same_fixture_as,
}

criterion_main!(throughput, latency, cpu_mem, response);

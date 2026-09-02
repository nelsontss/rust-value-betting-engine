use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};

use crate::domain::{
    Platform,
    entities::{
        Arbitrage, Game, Market, Odd,
        markets::{Line, total::TotalMarket},
    },
    services::cluster_service::ClusterService,
};

fn fixture_date(hour: u32, min: u32) -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
        NaiveTime::from_hms_milli_opt(hour, min, 0, 0).unwrap(),
    )
}

fn game(
    home_team: &str,
    away_team: &str,
    country: &str,
    competition: &str,
    hour: u32,
    min: u32,
    platform: Platform,
) -> Game {
    Game::new(
        home_team,
        away_team,
        country,
        competition,
        fixture_date(hour, min),
        platform,
        vec![],
    None,
    )
}

fn assert_cluster_sizes(cluster_service: &ClusterService, expected_sizes: &[usize]) {
    let total_clusters: usize = cluster_service
        .clusters
        .iter()
        .map(|clusters_by_key_ref| clusters_by_key_ref.value().iter().count())
        .sum();

    assert_eq!(expected_sizes.len(), total_clusters);

    let mut cluster_sizes: Vec<usize> = cluster_service
        .clusters
        .iter()
        .flat_map(|clusters_by_key_ref| {
            clusters_by_key_ref
                .value()
                .iter()
                .map(|cluster_ref| cluster_ref.game_count())
                .collect::<Vec<usize>>()
        })
        .collect();

    cluster_sizes.sort_unstable();

    assert_eq!(expected_sizes, cluster_sizes.as_slice());
}

fn fuzzy_portugal_game(home_team: &str, away_team: &str, platform: Platform) -> Game {
    Game::new(
        home_team,
        away_team,
        "Portugal",
        "Liga Portugal",
        fixture_date(15, 30),
        platform,
        vec![],
    None,
    )
}

fn fuzzy_england_game(home_team: &str, away_team: &str, platform: Platform) -> Game {
    Game::new(
        home_team,
        away_team,
        "England",
        "Premier League",
        fixture_date(15, 30),
        platform,
        vec![],
    None,
    )
}

fn porto_benfica(platform: Platform) -> Game {
    Game::new(
        "FC Porto",
        "SL Benfica",
        "Portugal",
        "Liga Portugal",
        fixture_date(15, 30),
        platform,
        vec![],
    None,
    )
}

fn sporting_braga(platform: Platform) -> Game {
    Game::new(
        "Sporting",
        "Braga",
        "Portugal",
        "Liga Portugal",
        fixture_date(17, 30),
        platform,
        vec![],
    None,
    )
}

fn arsenal_burnley(platform: Platform) -> Game {
    Game::new(
        "Arsenal",
        "Burnley",
        "England",
        "Premier League",
        fixture_date(18, 30),
        platform,
        vec![],
    None,
    )
}

fn porto_benfica_with_markets(platform: Platform, markets: Vec<Market>) -> Game {
    Game::new(
        "FC Porto",
        "SL Benfica",
        "Portugal",
        "Liga Portugal",
        fixture_date(15, 30),
        platform,
        markets,
        None,
    )
}

fn total_market(id: &str, line: f32, over: f64, under: f64) -> Market {
    Market::Total(TotalMarket::new(
        id.to_string(),
        Line(line),
        Odd::new(over).unwrap(),
        Odd::new(under).unwrap(),
    ))
}

#[test]
fn clusters_games_by_similarity_when_they_are_fully_equal() {
    let games = vec![
        porto_benfica(Platform::Betano),
        sporting_braga(Platform::Betano),
        arsenal_burnley(Platform::Betano),
        porto_benfica(Platform::Betano),
        porto_benfica(Platform::Betano),
        sporting_braga(Platform::Betano),
        arsenal_burnley(Platform::Betano),
    ];

    let cluster_service = ClusterService::new();
    cluster_service.add_games(games);

    assert_cluster_sizes(&cluster_service, &[2, 2, 3]);
}

#[test]
fn clusters_games_by_similarity_with_fuzzy_team_names() {
    let games = vec![
        fuzzy_portugal_game("FC Porto", "SL Benfica", Platform::Betano),
        fuzzy_portugal_game("Sporting CP", "Braga", Platform::Betano),
        fuzzy_england_game("Manchester Utd", "Arsenal", Platform::Betano),
        fuzzy_portugal_game("Porto FC", "Benfica SL", Platform::Betano),
        fuzzy_portugal_game("Porto", "Benfica", Platform::Betano),
        fuzzy_portugal_game("Sporting", "Braga", Platform::Betano),
        fuzzy_england_game("Man United", "Arsenal", Platform::Betano),
    ];

    let cluster_service = ClusterService::new();

    cluster_service.add_games(games);

    assert_cluster_sizes(&cluster_service, &[2, 2, 3]);
}

#[test]
fn clusters_games_by_similarity_with_fuzzy_competition_names() {
    let games = vec![
        game(
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portúgal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting",
            "Braga",
            "Portugal",
            "Liga Portugal",
            17,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting",
            "Braga",
            "Portugal",
            "liga portugal",
            17,
            30,
            Platform::Betano,
        ),
        game(
            "Arsenal",
            "Burnley",
            "England",
            "Premier League",
            18,
            30,
            Platform::Betano,
        ),
        game(
            "Arsenal",
            "Burnley",
            "England",
            "Prémier League",
            18,
            30,
            Platform::Betano,
        ),
    ];

    let cluster_service = ClusterService::new();

    cluster_service.add_games(games);

    assert_cluster_sizes(&cluster_service, &[2, 2, 2]);
}

#[test]
fn clusters_games_by_similarity_with_fuzzy_country_names() {
    let games = vec![
        game(
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "FC Porto",
            "SL Benfica",
            "Pórtugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting",
            "Braga",
            "Portugal",
            "Liga Portugal",
            17,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting",
            "Braga",
            "PORTUGAL",
            "Liga Portugal",
            17,
            30,
            Platform::Betano,
        ),
        game(
            "Arsenal",
            "Burnley",
            "England",
            "Premier League",
            18,
            30,
            Platform::Betano,
        ),
        game(
            "Arsenal",
            "Burnley",
            "Éngland",
            "Premier League",
            18,
            30,
            Platform::Betano,
        ),
    ];

    let cluster_service = ClusterService::new();

    cluster_service.add_games(games);

    assert_cluster_sizes(&cluster_service, &[2, 2, 2]);
}

#[test]
fn clusters_games_by_similarity_with_fuzzy_team_and_competition_names() {
    let games = vec![
        game(
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Porto FC",
            "Benfica SL",
            "Portugal",
            "Liga Portúgal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Porto",
            "Benfica",
            "Portugal",
            "liga portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting CP",
            "Braga",
            "Portugal",
            "Liga Portugal",
            17,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting",
            "Braga",
            "Portugal",
            "Liga Portúgal",
            17,
            30,
            Platform::Betano,
        ),
        game(
            "Manchester Utd",
            "Arsenal",
            "England",
            "Premier League",
            18,
            30,
            Platform::Betano,
        ),
        game(
            "Man United",
            "Arsenal",
            "England",
            "Prémier League",
            18,
            30,
            Platform::Betano,
        ),
    ];

    let cluster_service = ClusterService::new();

    cluster_service.add_games(games);

    assert_cluster_sizes(&cluster_service, &[2, 2, 3]);
}

#[test]
fn clusters_games_by_similarity_with_fuzzy_team_and_country_names() {
    let games = vec![
        game(
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Porto FC",
            "Benfica SL",
            "Pórtugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Porto",
            "Benfica",
            "PORTUGAL",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting CP",
            "Braga",
            "Portugal",
            "Liga Portugal",
            17,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting",
            "Braga",
            "PÓRTUGAL",
            "Liga Portugal",
            17,
            30,
            Platform::Betano,
        ),
        game(
            "Manchester Utd",
            "Arsenal",
            "England",
            "Premier League",
            18,
            30,
            Platform::Betano,
        ),
        game(
            "Man United",
            "Arsenal",
            "Éngland",
            "Premier League",
            18,
            30,
            Platform::Betano,
        ),
    ];

    let cluster_service = ClusterService::new();

    cluster_service.add_games(games);

    assert_cluster_sizes(&cluster_service, &[2, 2, 3]);
}

#[test]
fn clusters_games_by_similarity_with_combined_fuzzy_names() {
    let games = vec![
        game(
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Porto FC",
            "Benfica SL",
            "Pórtugal",
            "Liga Portúgal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Porto",
            "Benfica",
            "PORTUGAL",
            "liga portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting CP",
            "Braga",
            "Portugal",
            "Liga Portugal",
            17,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting",
            "Braga",
            "PÓRTUGAL",
            "Liga Portúgal",
            17,
            30,
            Platform::Betano,
        ),
        game(
            "Manchester Utd",
            "Arsenal",
            "England",
            "Premier League",
            18,
            30,
            Platform::Betano,
        ),
        game(
            "Man United",
            "Arsenal",
            "Éngland",
            "Prémier League",
            18,
            30,
            Platform::Betano,
        ),
    ];

    let cluster_service = ClusterService::new();

    cluster_service.add_games(games);

    assert_cluster_sizes(&cluster_service, &[2, 2, 3]);
}

#[test]
fn keeps_distinct_fixtures_separate_when_country_competition_and_date_match() {
    let games = vec![
        game(
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Porto FC",
            "Benfica SL",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting CP",
            "Braga",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Sporting",
            "Braga",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            Platform::Betano,
        ),
    ];

    let cluster_service = ClusterService::new();

    cluster_service.add_games(games);

    assert_cluster_sizes(&cluster_service, &[2, 2]);
}

#[test]
fn keeps_games_separate_when_only_one_team_side_matches() {
    let games = vec![
        game(
            "Manchester Utd",
            "Arsenal",
            "England",
            "Premier League",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Man United",
            "Chelsea",
            "England",
            "Premier League",
            15,
            30,
            Platform::Betano,
        ),
        game(
            "Manchester Utd",
            "Arsenal",
            "Éngland",
            "Prémier League",
            15,
            30,
            Platform::Betano,
        ),
    ];

    let cluster_service = ClusterService::new();

    cluster_service.add_games(games);

    assert_cluster_sizes(&cluster_service, &[1, 2]);
}

#[test]
fn insert_games_updates_existing_cluster_and_returns_new_arbitrage() {
    let first_game = porto_benfica_with_markets(
        Platform::Betano,
        vec![total_market("betano-total", 2.5, 2.15, 1.75)],
    );
    let second_game = porto_benfica_with_markets(Platform::Betano, vec![]);
    let second_game_id = second_game.id.clone();

    let cluster_service = ClusterService::new();

    cluster_service.add_games(vec![first_game, second_game.clone()]);

    let mut updated_second_game = second_game;

    updated_second_game.update_markets(vec![total_market("betclic-total", 2.5, 1.8, 2.15)]);

    let arbitrages = cluster_service.insert_games(vec![updated_second_game]);

    assert_eq!(1, arbitrages.len());
    assert!(matches!(arbitrages[0], Arbitrage::TwoWayLineArbitrage(_)));
    assert_cluster_sizes(&cluster_service, &[2]);
    assert!(
        cluster_service
            .game_id_to_fixture_cluster_key
            .contains_key(&second_game_id)
    );
}

#[test]
fn insert_games_adds_unknown_game_to_existing_cluster_and_returns_arbitrage() {
    let first_game = porto_benfica_with_markets(
        Platform::Betano,
        vec![total_market("betano-total", 2.5, 2.15, 1.75)],
    );
    let new_game = porto_benfica_with_markets(
        Platform::Betano,
        vec![total_market("betclic-total", 2.5, 1.8, 2.15)],
    );
    let new_game_id = new_game.id.clone();

    let cluster_service = ClusterService::new();

    cluster_service.add_games(vec![first_game]);

    let arbitrages = cluster_service.insert_games(vec![new_game]);

    assert_eq!(1, arbitrages.len());
    assert!(matches!(arbitrages[0], Arbitrage::TwoWayLineArbitrage(_)));
    assert_cluster_sizes(&cluster_service, &[2]);
    assert!(
        cluster_service
            .game_id_to_fixture_cluster_key
            .contains_key(&new_game_id)
    );
}

#[test]
fn insert_games_creates_new_cluster_for_unknown_distinct_fixture() {
    let first_game = porto_benfica(Platform::Betano);
    let new_game = sporting_braga(Platform::Betano);
    let new_game_id = new_game.id.clone();

    let cluster_service = ClusterService::new();

    cluster_service.add_games(vec![first_game]);

    let arbitrages = cluster_service.insert_games(vec![new_game]);

    assert!(arbitrages.is_empty());
    assert_cluster_sizes(&cluster_service, &[1, 1]);
    assert!(
        cluster_service
            .game_id_to_fixture_cluster_key
            .contains_key(&new_game_id)
    );
}
#[tokio::test]
async fn sweep_ended_clusters_removes_only_elapsed_fixtures() {
    let now = Utc::now().naive_utc();

    let past_betano = Game::new(
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        now - Duration::minutes(150),
        Platform::Betano,
        vec![],
    None,
    );
    let past_polymarket = Game::new(
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        now - Duration::minutes(150),
        Platform::Polymarket,
        vec![],
    None,
    );
    let future_game = Game::new(
        "Porto",
        "Braga",
        "Portugal",
        "Primeira Liga",
        now + Duration::hours(2),
        Platform::Betano,
        vec![],
    None,
    );

    let cluster_service = ClusterService::new();
    cluster_service.add_games(vec![past_betano.clone(), past_polymarket, future_game]);
    assert_cluster_sizes(&cluster_service, &[1, 2]);

    cluster_service.sweep_ended_clusters();

    // elapsed fixture removed from every index
    assert!(
        cluster_service
            .get_cluster(&past_betano.canonical_name())
            .is_err()
    );
    assert!(
        !cluster_service
            .game_id_to_fixture_cluster_key
            .contains_key(&past_betano.id)
    );
    assert!(
        !cluster_service
            .cluster_id_to_date
            .contains_key(&past_betano.canonical_name())
    );

    // upcoming fixtures remain untouched
    assert_cluster_sizes(&cluster_service, &[1]);
}

#[test]
fn get_clusters_only_returns_clusters_with_multiple_games() {
    let cluster_service = ClusterService::new();
    cluster_service.insert_games(vec![porto_benfica(Platform::Betano)]);
    cluster_service.insert_games(vec![porto_benfica(Platform::Polymarket)]);
    cluster_service.insert_games(vec![sporting_braga(Platform::Betano)]);

    let clusters = cluster_service.get_clusters();

    assert_eq!(1, clusters.len());
    assert_eq!(2, clusters[0].game_count());
}

#[test]
fn get_cluster_returns_stored_cluster_and_errors_for_unknown_id() {
    let cluster_service = ClusterService::new();
    let games = vec![
        porto_benfica(Platform::Betano),
        porto_benfica(Platform::Polymarket),
    ];
    cluster_service.insert_games(games);

    let game_id = cluster_service.get_games()[0].id.clone();
    let cluster_key = cluster_service.game_id_to_fixture_cluster_key.get(&game_id).unwrap().clone();

    let cluster = match cluster_service.get_cluster(&cluster_key) {
        Ok(cluster) => cluster,
        Err(_) => panic!("expected to find the stored cluster"),
    };
    assert_eq!(2, cluster.game_count());

    assert!(matches!(
        cluster_service.get_cluster("unknown-key"),
        Err(ClusterServiceErrors::ClusterNotFound)
    ));
}

#[test]
fn get_games_returns_every_game_across_clusters() {
    let cluster_service = ClusterService::new();
    cluster_service.insert_games(vec![
        porto_benfica(Platform::Betano),
        porto_benfica(Platform::Polymarket),
        sporting_braga(Platform::Betano),
    ]);

    let games = cluster_service.get_games();

    assert_eq!(3, games.len());
    let platforms: Vec<Platform> = games.iter().map(|g| g.platform()).collect();
    assert!(platforms.contains(&Platform::Betano));
    assert!(platforms.contains(&Platform::Polymarket));
}

#[test]
fn get_plaftorm_games_filters_by_platform() {
    let cluster_service = ClusterService::new();
    cluster_service.insert_games(vec![
        porto_benfica(Platform::Betano),
        porto_benfica(Platform::Polymarket),
        sporting_braga(Platform::Bwin),
    ]);

    let betano_games = cluster_service.get_plaftorm_games(&Platform::Betano);
    assert_eq!(1, betano_games.len());
    assert_eq!(Platform::Betano, betano_games[0].platform());

    let bwin_games = cluster_service.get_plaftorm_games(&Platform::Bwin);
    assert_eq!(1, bwin_games.len());

    let polymarket_games = cluster_service.get_plaftorm_games(&Platform::Polymarket);
    assert_eq!(1, polymarket_games.len());
}

#[test]
fn insert_games_ignores_games_already_clustered_by_id() {
    let cluster_service = ClusterService::new();
    let first = porto_benfica(Platform::Betano);
    let same_id = {
        let mut game = porto_benfica(Platform::Polymarket);
        game.id = first.id.clone();
        game
    };

    cluster_service.insert_games(vec![first]);
    cluster_service.insert_games(vec![same_id]);

    // the polymarket twin shares the id of the first game, so it must be skipped
    // entirely rather than being added as a new game
    assert_eq!(1, cluster_service.get_games().len());
}

#[test]
fn insert_markets_updates_existing_game_and_returns_arbitrage() {
    let cluster_service = ClusterService::new();
    cluster_service.insert_games(vec![porto_benfica(Platform::Betano)]);
    cluster_service.insert_games(vec![porto_benfica(Platform::Polymarket)]);

    let game_id = cluster_service
        .get_plaftorm_games(&Platform::Betano)[0]
        .id
        .clone();

    // over 1.8 + under 2.0 sums above one: no arbitrage
    let arbitrage = cluster_service.insert_markets(
        &game_id,
        vec![total_market("total-1", 2.5, 1.8, 2.0)],
    );
    assert!(arbitrage.is_empty());

    // updating an unknown game id must be a no-op
    let unknown = cluster_service.insert_markets(
        "unknown-game",
        vec![total_market("total-1", 2.5, 1.8, 2.0)],
    );
    assert!(unknown.is_empty());
}

#[test]
fn insert_games_returns_arbitrage_when_two_bookmakers_complete_a_market() {
    let cluster_service = ClusterService::new();
    cluster_service.insert_games(vec![porto_benfica_with_markets(
        Platform::Betano,
        vec![total_market("total-betano", 2.5, 2.2, 1.7)],
    )]);

    let polymarket = porto_benfica_with_markets(
        Platform::Polymarket,
        vec![total_market("total-poly", 2.5, 1.7, 2.2)],
    );

    // best over 2.2 + best under 2.2 across platforms is a guaranteed-profit pair
    let arbitrage = cluster_service.insert_games(vec![polymarket]);

    assert_eq!(1, arbitrage.len());
    assert!((arbitrage[0].roi() - 0.1).abs() < 1e-6);
}

#[test]
fn display_writes_one_line_per_cluster() {
    let cluster_service = ClusterService::new();
    cluster_service.insert_games(vec![porto_benfica(Platform::Betano)]);
    cluster_service.insert_games(vec![porto_benfica(Platform::Polymarket)]);

    let display = cluster_service.to_string();

    assert!(display.contains("benfica") && display.contains("porto"));
}

#[tokio::test]
async fn subscribe_to_cluster_updates_receives_updates_on_cluster_changes() {
    use std::sync::Arc as StdArc;
    use std::time::Duration;

    let cluster_service = StdArc::new(ClusterService::new());
    let mut rx = cluster_service.subscribe_to_cluster_updates();

    cluster_service.insert_games(vec![porto_benfica(Platform::Betano)]);
    // no event yet: single-game clusters are not broadcast
    assert!(
        tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .is_err()
    );

    cluster_service.insert_games(vec![porto_benfica(Platform::Polymarket)]);
    let update = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("cluster update should be broadcast")
        .unwrap();

    assert_eq!(2, update.game_count());
}

use super::ClusterServiceErrors;

#[tokio::test]
async fn sweep_ended_clusters_closes_and_persists_expired_fixtures() {
    use std::sync::Arc as StdArc;

    use crate::domain::services::statistics_service::StatisticsService;
    use crate::infrastructure::repositories::{
        connect_pool,
        fixture_cluster_repository::FixtureClusterRepository,
        game_repository::GameRepository,
    };

    let db_path = format!(
        "{}/cluster_service_sweep_test_{}.db",
        std::env::temp_dir().display(),
        uuid::Uuid::new_v4()
    );
    let pool = connect_pool(&db_path).await.unwrap();
    let game_repo = StdArc::new(GameRepository::from_pool(pool.clone()));
    let cluster_repo = StdArc::new(FixtureClusterRepository::from_pool(pool, game_repo));
    cluster_repo.run_migrations().await.unwrap();
    let statistics_service = StdArc::new(StatisticsService::new(cluster_repo.clone()));

    let cluster_service = StdArc::new(
        ClusterService::new()
            .with_fixture_cluster_repository(cluster_repo.clone())
            .with_statistics_service(statistics_service.clone()),
    );

    // a game that started more than the grace period ago
    let elapsed = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap(),
    );
    let mut game = Game::new(
        "FC Porto",
        "SL Benfica",
        "Portugal",
        "Liga Portugal",
        elapsed,
        Platform::Betano,
        vec![total_market("total-1", 2.5, 1.9, 1.9)],
    None,
    );
    game.id = "elapsed-game".to_string();
    let mut poly = Game::new(
        "FC Porto",
        "SL Benfica",
        "Portugal",
        "Liga Portugal",
        elapsed,
        Platform::Polymarket,
        vec![Market::Total(TotalMarket::new(
            "total-1".to_string(),
            Line(2.5),
            Odd::new_from_prob(
                polymarket_client_sdk_v2::types::dec!(0.55),
                polymarket_client_sdk_v2::types::dec!(0.45),
            )
            .unwrap(),
            Odd::new_from_prob(
                polymarket_client_sdk_v2::types::dec!(0.45),
                polymarket_client_sdk_v2::types::dec!(0.55),
            )
            .unwrap(),
        ))],
    None,
    );
    poly.id = "elapsed-poly".to_string();

    cluster_service.insert_games(vec![game]);
    cluster_service.insert_games(vec![poly]);
    assert_eq!(1, cluster_service.get_clusters().len());

    // simulate the delayed persist task having already flushed the
    // intermediate (open) snapshot, as it would in a long-running process
    cluster_service.pending_persists.clear();

    cluster_service.sweep_ended_clusters();

    // cluster was removed from memory and marked closed in the pending persists
    assert!(cluster_service.get_clusters().is_empty());
    assert!(cluster_service.get_games().is_empty());

    // flush pushes the closed cluster into the database
    cluster_service.flush_pending_persists().await;
    let persisted = cluster_repo.get_all_clusters().await.unwrap();
    assert_eq!(1, persisted.len());
    assert!(persisted[0].is_closed());
}

#[tokio::test]
async fn flush_pending_persists_without_repository_is_a_no_op() {
    let cluster_service = ClusterService::new();
    cluster_service.insert_games(vec![porto_benfica(Platform::Betano)]);

    // must not panic without a configured repository
    cluster_service.flush_pending_persists().await;
}

#[test]
fn persist_helpers_skip_when_called_outside_a_tokio_runtime() {
    use std::sync::Arc as StdArc;

    use crate::domain::entities::FixtureCluster;
    use crate::infrastructure::repositories::{
        connect_pool,
        fixture_cluster_repository::FixtureClusterRepository,
        game_repository::GameRepository,
    };

    let repo = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap()
        .block_on(async {
            let db_path = format!(
                "{}/cluster_service_persist_test_{}.db",
                std::env::temp_dir().display(),
                uuid::Uuid::new_v4()
            );
            let pool = connect_pool(&db_path).await.unwrap();
            let game_repo = StdArc::new(GameRepository::from_pool(pool.clone()));
            let cluster_repo = StdArc::new(FixtureClusterRepository::from_pool(pool, game_repo));
            cluster_repo.run_migrations().await.unwrap();
            cluster_repo
        });
    let service = StdArc::new(ClusterService::new().with_fixture_cluster_repository(repo));
    assert!(service.fixture_cluster_repository.is_some());

    let cluster = StdArc::new(FixtureCluster::new(porto_benfica(Platform::Betano)));
    let cluster_for_thread = cluster;
    let service_for_thread = service.clone();
    let handle = std::thread::spawn(move || {
        // both helpers warn and return when there is no runtime context
        service_for_thread.persist_cluster(cluster_for_thread);
        service_for_thread.persist_cluster_diffs("k".to_string(), Default::default());
    });
    handle.join().unwrap();

    assert!(service.pending_persists.is_empty());
}

#[test]
fn end_cluster_ignores_unknown_cluster_ids() {
    let cluster_service = ClusterService::new();

    // must be a no-op rather than a panic
    cluster_service.end_cluster("does-not-exist");
}

#[tokio::test]
async fn end_of_game_sweeper_ticks_without_clusters() {
    use std::sync::Arc as StdArc;
    use std::time::Duration;

    let cluster_service = StdArc::new(ClusterService::new());
    cluster_service.start_end_of_game_sweeper();

    // let the first (immediate) interval tick run
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(cluster_service.get_clusters().is_empty());
}

#[test]
fn add_games_skips_duplicate_ids_within_one_batch() {
    let cluster_service = ClusterService::new();
    let game = porto_benfica(Platform::Betano);
    let mut duplicate = porto_benfica(Platform::Polymarket);
    duplicate.id = game.id.clone();

    cluster_service.insert_games(vec![game, duplicate]);

    assert_eq!(1, cluster_service.get_games().len());
}

#[tokio::test]
async fn insert_games_updates_existing_cluster_and_broadcasts_when_multi_game() {
    use std::sync::Arc as StdArc;
    use std::time::Duration;

    use crate::domain::services::market_service::MarketService;

    let cluster_service = StdArc::new(ClusterService::new());
    let mut rx = cluster_service.subscribe_to_cluster_updates();

    cluster_service.insert_games(vec![porto_benfica(Platform::Betano)]);
    cluster_service.insert_games(vec![porto_benfica(Platform::Polymarket)]);

    // drain the broadcast from the second insert
    let _ = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("first update should be broadcast");

    // re-insert the betano game with fresh markets: update path
    let game_id = cluster_service
        .get_plaftorm_games(&Platform::Betano)[0]
        .id
        .clone();
    let mut updated = porto_benfica(Platform::Betano);
    updated.id = game_id.clone();
    updated.update_markets(vec![total_market("total-updated", 2.5, 1.9, 1.9)]);

    cluster_service.insert_games(vec![updated]);

    let update = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("updated cluster should be broadcast")
        .unwrap();
    assert_eq!(2, update.game_count());
}

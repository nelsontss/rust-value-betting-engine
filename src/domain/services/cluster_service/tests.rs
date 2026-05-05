use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::domain::{entities::Game, services::cluster_service::ClusterService};

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
    platform: &str,
) -> Game {
    Game::new(
        home_team,
        away_team,
        country,
        competition,
        fixture_date(hour, min),
        platform,
    )
}

fn assert_cluster_sizes(cluster_service: &ClusterService, expected_sizes: &[usize]) {
    let total_clusters: usize = cluster_service
        .clusters
        .values()
        .map(|clusters_on_date| clusters_on_date.len())
        .sum();

    assert_eq!(expected_sizes.len(), total_clusters);

    let mut cluster_sizes: Vec<usize> = cluster_service
        .clusters
        .values()
        .flat_map(|clusters_on_date| clusters_on_date.iter())
        .map(|cluster| cluster.game_count())
        .collect();

    cluster_sizes.sort_unstable();

    assert_eq!(expected_sizes, cluster_sizes.as_slice());
}

fn fuzzy_portugal_game(home_team: &str, away_team: &str, platform: &str) -> Game {
    Game::new(
        home_team,
        away_team,
        "Portugal",
        "Liga Portugal",
        fixture_date(15, 30),
        platform,
    )
}

fn fuzzy_england_game(home_team: &str, away_team: &str, platform: &str) -> Game {
    Game::new(
        home_team,
        away_team,
        "England",
        "Premier League",
        fixture_date(15, 30),
        platform,
    )
}

fn porto_benfica(platform: &str) -> Game {
    Game::new(
        "FC Porto",
        "SL Benfica",
        "Portugal",
        "Liga Portugal",
        fixture_date(15, 30),
        platform,
    )
}

fn sporting_braga(platform: &str) -> Game {
    Game::new(
        "Sporting",
        "Braga",
        "Portugal",
        "Liga Portugal",
        fixture_date(17, 30),
        platform,
    )
}

fn arsenal_burnley(platform: &str) -> Game {
    Game::new(
        "Arsenal",
        "Burnley",
        "England",
        "Premier League",
        fixture_date(18, 30),
        platform,
    )
}

#[test]
fn clusters_games_by_similarity_when_they_are_fully_equal() {
    let games = vec![
        porto_benfica("Betano"),
        sporting_braga("Betano"),
        arsenal_burnley("Betano"),
        porto_benfica("Betclic"),
        porto_benfica("22Bet"),
        sporting_braga("Betclic"),
        arsenal_burnley("Betclic"),
    ];

    let cluster_service = ClusterService::new(&games);

    assert_cluster_sizes(&cluster_service, &[2, 2, 3]);
}

#[test]
fn clusters_games_by_similarity_with_fuzzy_team_names() {
    let games = vec![
        fuzzy_portugal_game("FC Porto", "SL Benfica", "Betano"),
        fuzzy_portugal_game("Sporting CP", "Braga", "Betano"),
        fuzzy_england_game("Manchester Utd", "Arsenal", "Betano"),
        fuzzy_portugal_game("Porto FC", "Benfica SL", "Betclic"),
        fuzzy_portugal_game("Porto", "Benfica", "22Bet"),
        fuzzy_portugal_game("Sporting", "Braga", "Betclic"),
        fuzzy_england_game("Man United", "Arsenal", "Betclic"),
    ];

    let cluster_service = ClusterService::new(&games);

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
            "Betano",
        ),
        game(
            "FC Porto",
            "SL Benfica",
            "Portugal",
            "Liga Portúgal",
            15,
            30,
            "Betclic",
        ),
        game(
            "Sporting",
            "Braga",
            "Portugal",
            "Liga Portugal",
            17,
            30,
            "Betano",
        ),
        game(
            "Sporting",
            "Braga",
            "Portugal",
            "liga portugal",
            17,
            30,
            "Betclic",
        ),
        game(
            "Arsenal",
            "Burnley",
            "England",
            "Premier League",
            18,
            30,
            "Betano",
        ),
        game(
            "Arsenal",
            "Burnley",
            "England",
            "Prémier League",
            18,
            30,
            "Betclic",
        ),
    ];

    let cluster_service = ClusterService::new(&games);

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
            "Betano",
        ),
        game(
            "FC Porto",
            "SL Benfica",
            "Pórtugal",
            "Liga Portugal",
            15,
            30,
            "Betclic",
        ),
        game(
            "Sporting",
            "Braga",
            "Portugal",
            "Liga Portugal",
            17,
            30,
            "Betano",
        ),
        game(
            "Sporting",
            "Braga",
            "PORTUGAL",
            "Liga Portugal",
            17,
            30,
            "Betclic",
        ),
        game(
            "Arsenal",
            "Burnley",
            "England",
            "Premier League",
            18,
            30,
            "Betano",
        ),
        game(
            "Arsenal",
            "Burnley",
            "Éngland",
            "Premier League",
            18,
            30,
            "Betclic",
        ),
    ];

    let cluster_service = ClusterService::new(&games);

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
            "Betano",
        ),
        game(
            "Porto FC",
            "Benfica SL",
            "Portugal",
            "Liga Portúgal",
            15,
            30,
            "Betclic",
        ),
        game(
            "Porto",
            "Benfica",
            "Portugal",
            "liga portugal",
            15,
            30,
            "22Bet",
        ),
        game(
            "Sporting CP",
            "Braga",
            "Portugal",
            "Liga Portugal",
            17,
            30,
            "Betano",
        ),
        game(
            "Sporting",
            "Braga",
            "Portugal",
            "Liga Portúgal",
            17,
            30,
            "Betclic",
        ),
        game(
            "Manchester Utd",
            "Arsenal",
            "England",
            "Premier League",
            18,
            30,
            "Betano",
        ),
        game(
            "Man United",
            "Arsenal",
            "England",
            "Prémier League",
            18,
            30,
            "Betclic",
        ),
    ];

    let cluster_service = ClusterService::new(&games);

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
            "Betano",
        ),
        game(
            "Porto FC",
            "Benfica SL",
            "Pórtugal",
            "Liga Portugal",
            15,
            30,
            "Betclic",
        ),
        game(
            "Porto",
            "Benfica",
            "PORTUGAL",
            "Liga Portugal",
            15,
            30,
            "22Bet",
        ),
        game(
            "Sporting CP",
            "Braga",
            "Portugal",
            "Liga Portugal",
            17,
            30,
            "Betano",
        ),
        game(
            "Sporting",
            "Braga",
            "PÓRTUGAL",
            "Liga Portugal",
            17,
            30,
            "Betclic",
        ),
        game(
            "Manchester Utd",
            "Arsenal",
            "England",
            "Premier League",
            18,
            30,
            "Betano",
        ),
        game(
            "Man United",
            "Arsenal",
            "Éngland",
            "Premier League",
            18,
            30,
            "Betclic",
        ),
    ];

    let cluster_service = ClusterService::new(&games);

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
            "Betano",
        ),
        game(
            "Porto FC",
            "Benfica SL",
            "Pórtugal",
            "Liga Portúgal",
            15,
            30,
            "Betclic",
        ),
        game(
            "Porto",
            "Benfica",
            "PORTUGAL",
            "liga portugal",
            15,
            30,
            "22Bet",
        ),
        game(
            "Sporting CP",
            "Braga",
            "Portugal",
            "Liga Portugal",
            17,
            30,
            "Betano",
        ),
        game(
            "Sporting",
            "Braga",
            "PÓRTUGAL",
            "Liga Portúgal",
            17,
            30,
            "Betclic",
        ),
        game(
            "Manchester Utd",
            "Arsenal",
            "England",
            "Premier League",
            18,
            30,
            "Betano",
        ),
        game(
            "Man United",
            "Arsenal",
            "Éngland",
            "Prémier League",
            18,
            30,
            "Betclic",
        ),
    ];

    let cluster_service = ClusterService::new(&games);

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
            "Betano",
        ),
        game(
            "Porto FC",
            "Benfica SL",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            "Betclic",
        ),
        game(
            "Sporting CP",
            "Braga",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            "Betano",
        ),
        game(
            "Sporting",
            "Braga",
            "Portugal",
            "Liga Portugal",
            15,
            30,
            "Betclic",
        ),
    ];

    let cluster_service = ClusterService::new(&games);

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
            "Betano",
        ),
        game(
            "Man United",
            "Chelsea",
            "England",
            "Premier League",
            15,
            30,
            "Betclic",
        ),
        game(
            "Manchester Utd",
            "Arsenal",
            "Éngland",
            "Prémier League",
            15,
            30,
            "22Bet",
        ),
    ];

    let cluster_service = ClusterService::new(&games);

    assert_cluster_sizes(&cluster_service, &[1, 2]);
}

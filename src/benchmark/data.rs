use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::domain::{
    entities::{Game, Market},
    services::ClusterService,
};

const PLATFORMS: &[&str] = &[
    "Betano", "Betclic", "22Bet", "Sportingbet", "Bwin", "Moosh", "Solverde",
    "Luckia", "Esc Online", "Placard",
];

const TEAMS: &[&str] = &[
    "FC Porto",
    "SL Benfica",
    "Sporting CP",
    "SC Braga",
    "Vitória SC",
    "Manchester United",
    "Manchester City",
    "Liverpool",
    "Arsenal",
    "Chelsea",
    "Tottenham",
    "Newcastle",
    "Aston Villa",
    "FC Barcelona",
    "Real Madrid",
    "Atlético Madrid",
    "Real Sociedad",
    "Athletic Bilbao",
    "Real Betis",
    "Villarreal",
    "Sevilla",
    "Juventus",
    "Inter Milan",
    "AC Milan",
    "Napoli",
    "AS Roma",
    "Lazio",
    "Atalanta",
    "Fiorentina",
    "Bayern Munich",
    "Borussia Dortmund",
    "RB Leipzig",
    "Bayer Leverkusen",
    "Eintracht Frankfurt",
    "VfB Stuttgart",
    "Wolfsburg",
    "SC Freiburg",
    "Paris Saint-Germain",
    "Olympique Marseille",
    "AS Monaco",
    "Olympique Lyonnais",
    "Lille",
    "OGC Nice",
    "Stade Rennais",
    "RC Lens",
    "Ajax",
    "Feyenoord",
    "PSV Eindhoven",
    "AZ Alkmaar",
    "FC Twente",
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

fn fixture_date(day_offset: u32, hour: u32, min: u32) -> NaiveDateTime {
    let base = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let date = base + chrono::Duration::days(day_offset as i64);
    NaiveDateTime::new(date, NaiveTime::from_hms_milli_opt(hour.min(23), min, 0, 0).unwrap())
}

/// Generate `count` games that are all distinct fixtures (different teams, countries,
/// competitions, dates). No two games will cluster together.
pub fn generate_distinct_fixtures(count: usize) -> Vec<Game> {
    (0..count)
        .map(|i| {
            let day = i as u32 / 50;
            let hour = 15 + (i % 6) as u32;
            let home_idx = i % TEAMS.len();
            let away_idx = (i + 1 + i / TEAMS.len()) % TEAMS.len();
            let country_idx = i % COUNTRIES.len();
            let competition_idx = i % COMPETITIONS.len();

            Game::new(
                TEAMS[home_idx],
                TEAMS[away_idx],
                COUNTRIES[country_idx],
                COMPETITIONS[competition_idx],
                fixture_date(day, hour.min(23), 30),
                PLATFORMS[i % PLATFORMS.len()],
                vec![],
            )
        })
        .collect()
}

/// Generate `platform_count` games for the same fixture (same teams, country, competition,
/// date) across different platforms — they will all cluster together.
pub fn generate_same_fixture_with_platforms(
    home_team: &str,
    away_team: &str,
    country: &str,
    competition: &str,
    date: NaiveDateTime,
    platform_count: usize,
    with_markets: bool,
) -> Vec<Game> {
    (0..platform_count)
        .map(|i| {
            let mut markets = vec![];
            if with_markets {
                markets = vec![
                    Market::total(
                        &format!("total-{}", i),
                        2.5,
                        1.85 + (i as f64 * 0.02),
                        1.95 - (i as f64 * 0.01),
                    )
                    .unwrap(),
                ];
            }
            Game::new(
                home_team,
                away_team,
                country,
                competition,
                date,
                PLATFORMS[i % PLATFORMS.len()],
                markets,
            )
        })
        .collect()
}

/// Generate a cluster with `platform_count` variations of the same fixture.
pub fn generate_cluster(
    home_team: &str,
    away_team: &str,
    platform_count: usize,
    with_markets: bool,
) -> Vec<Game> {
    let date = fixture_date(0, 15, 30);
    generate_same_fixture_with_platforms(
        home_team,
        away_team,
        "Portugal",
        "Liga Portugal",
        date,
        platform_count,
        with_markets,
    )
}

/// Generate `cluster_count` clusters, each with `platforms_per_cluster` games.
/// Returns all games flattened. Total = cluster_count * platforms_per_cluster.
pub fn generate_many_clusters(
    cluster_count: usize,
    platforms_per_cluster: usize,
    with_markets: bool,
) -> Vec<Game> {
    let mut games = Vec::with_capacity(cluster_count * platforms_per_cluster);

    for cluster_idx in 0..cluster_count {
        let day = cluster_idx as u32 / 10;
        let hour = 10 + (cluster_idx % 10) as u32;
        let date = fixture_date(day, hour.min(23), 0);
        let home_idx = cluster_idx % TEAMS.len();
        let away_idx = (cluster_idx + 1 + cluster_idx / TEAMS.len()) % TEAMS.len();

        for platform_idx in 0..platforms_per_cluster {
            let mut markets = vec![];
            if with_markets {
                markets = vec![
                    Market::total(
                        &format!("t-{}-{}", cluster_idx, platform_idx),
                        2.5,
                        1.85 + (platform_idx as f64 * 0.02),
                        1.95 - (platform_idx as f64 * 0.01),
                    )
                    .unwrap(),
                ];
            }
            games.push(Game::new(
                TEAMS[home_idx],
                TEAMS[away_idx],
                COUNTRIES[cluster_idx % COUNTRIES.len()],
                COMPETITIONS[cluster_idx % COMPETITIONS.len()],
                date,
                PLATFORMS[platform_idx % PLATFORMS.len()],
                markets,
            ));
        }
    }

    games
}

/// Generate games with match result markets that create arbitrage opportunities.
pub fn generate_arbitrage_games(
    count: usize,
    home_team: &str,
    away_team: &str,
    country: &str,
    competition: &str,
) -> Vec<Game> {
    (0..count)
        .map(|i| {
            let home_odd = 2.0 + (i as f64 * 0.15);
            let away_odd = 2.0 + (i as f64 * 0.12);
            let draw_odd = 3.0 + (i as f64 * 0.1);

            let markets = vec![
                Market::match_result(&format!("mr-{}", i), home_odd, draw_odd, away_odd).unwrap(),
            ];

            Game::new(
                home_team,
                away_team,
                country,
                competition,
                fixture_date(0, 15, 30),
                PLATFORMS[i % PLATFORMS.len()],
                markets,
            )
        })
        .collect()
}

/// Generate games that cover all 5 market types.
pub fn generate_all_market_types(
    platform: &str,
    game_label: &str,
) -> Game {
    let markets = vec![
        Market::match_result(&format!("{}-mr", game_label), 2.5, 3.2, 2.8).unwrap(),
        Market::moneyline(&format!("{}-ml", game_label), 1.8, 2.0).unwrap(),
        Market::total(&format!("{}-t2.5", game_label), 2.5, 1.9, 1.9).unwrap(),
        Market::handicap(&format!("{}-h1", game_label), 1.0, 2.1, 3.3, 3.5).unwrap(),
        Market::asian_handicap(&format!("{}-ah", game_label), -0.5, 2.0, 1.85).unwrap(),
    ];

    Game::new(
        "FC Porto",
        "SL Benfica",
        "Portugal",
        "Liga Portugal",
        fixture_date(0, 15, 30),
        platform,
        markets,
    )
}

/// Build a ClusterService pre-loaded with `game_count` distinct fixtures.
pub fn build_loaded_service(game_count: usize) -> ClusterService {
    let games = generate_distinct_fixtures(game_count);
    ClusterService::new(games)
}

/// Build a ClusterService pre-loaded with `cluster_count` clusters,
/// each having `platforms_per_cluster` games.
pub fn build_loaded_service_with_clusters(
    cluster_count: usize,
    platforms_per_cluster: usize,
) -> ClusterService {
    let games = generate_many_clusters(cluster_count, platforms_per_cluster, true);
    ClusterService::new(games)
}

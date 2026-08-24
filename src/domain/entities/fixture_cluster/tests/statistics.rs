use std::collections::HashMap;

use chrono::Utc;

use crate::domain::entities::Outcome;

use super::*;

#[test]
fn statistics_diffs_pairs_polymarket_probs_with_other_platforms_median() {
    let poly_game = game_with_markets(
        Platform::Polymarket,
        vec![total_market("poly-total", 2.5, 2.0, 2.0)],
    );
    let betano_game = game_with_markets(
        Platform::Betano,
        vec![total_market("betano-total", 2.5, 1.9, 2.1)],
    );

    let mut cluster = FixtureCluster::new(poly_game);
    assert!(cluster.try_to_add_game(betano_game).is_ok());

    let diffs = cluster.live_statistics_diffs();
    let total_type = total_market_type(2.5);

    let over_diff = diffs[&(total_type.clone(), Outcome::Over)];
    let under_diff = diffs[&(total_type, Outcome::Under)];

    let poly_over = 1.0 / 2.0;
    let poly_under = 1.0 / 2.0;
    let betano_over = 1.0 / 1.9;
    let betano_under = 1.0 / 2.1;

    assert!((over_diff - (poly_over - betano_over)).abs() < 1e-9);
    assert!((under_diff - (poly_under - betano_under)).abs() < 1e-9);
}

#[test]
fn statistics_diffs_are_empty_without_polymarket_game() {
    let betano_game = game_with_markets(
        Platform::Betano,
        vec![total_market("betano-total", 2.5, 1.9, 2.1)],
    );
    let bwin_game = game_with_markets(
        Platform::Bwin,
        vec![total_market("bwin-total", 2.5, 2.0, 2.0)],
    );

    let mut cluster = FixtureCluster::new(betano_game);
    assert!(cluster.try_to_add_game(bwin_game).is_ok());

    assert!(cluster.live_statistics_diffs().is_empty());
}

#[test]
fn statistics_diffs_are_empty_without_other_platforms() {
    let poly_game = game_with_markets(
        Platform::Polymarket,
        vec![total_market("poly-total", 2.5, 2.0, 2.0)],
    );

    let cluster = FixtureCluster::new(poly_game);

    assert!(cluster.live_statistics_diffs().is_empty());
}

#[test]
fn statistics_diffs_averages_accumulated_ticks() {
    let poly_game = game_with_markets(
        Platform::Polymarket,
        vec![total_market("poly-total", 2.5, 2.0, 2.0)],
    );
    let poly_id = poly_game.id.clone();
    let betano_game = game_with_markets(
        Platform::Betano,
        vec![total_market("betano-total", 2.5, 1.9, 2.1)],
    );

    let mut cluster = FixtureCluster::new(poly_game);
    assert!(cluster.try_to_add_game(betano_game).is_ok());

    let total_type = total_market_type(2.5);
    let tick1_over = 0.5 - 1.0 / 1.9;
    let tick1_under = 0.5 - 1.0 / 2.1;

    let updated_poly = total_market("poly-total", 2.5, 1.8, 2.2).1;
    cluster.update_markets(&poly_id, vec![updated_poly]);

    let tick2_over = 1.0 / 1.8 - 1.0 / 1.9;
    let tick2_under = 1.0 / 2.2 - 1.0 / 2.1;

    let means = cluster.statistics_diffs();

    assert_eq!(2, cluster.diffs[&(total_type.clone(), Outcome::Over)].len());
    assert!(
        (means[&(total_type.clone(), Outcome::Over)] - (tick1_over + tick2_over) / 2.0).abs()
            < 1e-9
    );
    assert!(
        (means[&(total_type.clone(), Outcome::Under)] - (tick1_under + tick2_under) / 2.0).abs()
            < 1e-9
    );

    // live diff reflects the latest tick only
    let live = cluster.live_statistics_diffs();
    assert!((live[&(total_type, Outcome::Over)] - tick2_over).abs() < 1e-9);
}

#[test]
fn from_persisted_uses_saved_mean_diffs_and_does_not_fabricate_ticks() {
    let games = vec![
        game_with_markets(
            Platform::Betano,
            vec![moneyline_market("betano-moneyline", 2.0, 1.8)],
        ),
        game_with_markets(
            Platform::Polymarket,
            vec![moneyline_market("poly-moneyline", 2.1, 1.7)],
        ),
    ];

    let mut mean_diffs = HashMap::new();
    mean_diffs.insert((MarketType::Moneyline, Outcome::Home), 0.05);
    mean_diffs.insert((MarketType::Moneyline, Outcome::Away), -0.03);

    let completed = FixtureCluster::from_persisted(
        "benfica-sporting".to_string(),
        games.clone(),
        Utc::now(),
        mean_diffs.clone(),
        false,
    );

    assert_eq!(mean_diffs, completed.statistics_diffs());
    assert!(!completed.is_closed());

    let live_only = FixtureCluster::from_persisted(
        "benfica-sporting".to_string(),
        games,
        Utc::now(),
        HashMap::new(),
        false,
    );

    // no fabricated ticks: statistics empty until diffs are persisted for it
    assert!(live_only.statistics_diffs().is_empty());
    assert!(!live_only.live_statistics_diffs().is_empty());
}

#[test]
fn from_persisted_restores_closed_fixtures_as_closed() {
    let games = vec![game_with_markets(
        Platform::Betano,
        vec![moneyline_market("betano-moneyline", 2.0, 1.8)],
    )];

    let cluster = FixtureCluster::from_persisted(
        "benfica-sporting".to_string(),
        games,
        Utc::now(),
        HashMap::new(),
        true,
    );

    assert!(cluster.is_closed());
}

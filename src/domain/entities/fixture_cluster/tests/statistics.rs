use std::collections::HashMap;

use chrono::Utc;
use rust_decimal::Decimal;

use crate::domain::entities::Outcome;

use super::*;

fn poly_odd(prob: f64, prob_no: f64) -> Odd {
    Odd::new_from_prob(
        Decimal::try_from(prob).unwrap(),
        Decimal::try_from(prob_no).unwrap(),
    )
    .unwrap()
}

fn poly_total_market(
    id: &str,
    line: f32,
    over_prob: f64,
    over_prob_no: f64,
    under_prob: f64,
    under_prob_no: f64,
) -> (MarketType, Market) {
    (
        total_market_type(line),
        Market::Total(TotalMarket::new(
            id.to_string(),
            Line(line),
            poly_odd(over_prob, over_prob_no),
            poly_odd(under_prob, under_prob_no),
        )),
    )
}

fn poly_moneyline_market(
    id: &str,
    home_prob: f64,
    home_prob_no: f64,
    away_prob: f64,
    away_prob_no: f64,
) -> (MarketType, Market) {
    (
        MarketType::Moneyline,
        Market::Moneyline(MoneylineMarket::new(
            id.to_string(),
            poly_odd(home_prob, home_prob_no),
            poly_odd(away_prob, away_prob_no),
        )),
    )
}

#[test]
fn statistics_diffs_pairs_polymarket_probs_with_other_platforms_median() {
    let poly_game = game_with_markets(
        Platform::Polymarket,
        vec![poly_total_market("poly-total", 2.5, 0.5, 0.6, 0.5, 0.7)],
    );
    let betano_game = game_with_markets(
        Platform::Betano,
        vec![total_market("betano-total", 2.5, 1.9, 2.1)],
    );

    let mut cluster = FixtureCluster::new(poly_game);
    assert!(cluster.try_to_add_game(betano_game).is_ok());

    let diffs = cluster.live_statistics_diffs();
    let total_type = total_market_type(2.5);

    let (over_diff, over_diff_from_no) = diffs[&total_type][&Outcome::Over];
    let (under_diff, under_diff_from_no) = diffs[&total_type][&Outcome::Under];

    let betano_over = 1.0 / 1.9;
    let betano_under = 1.0 / 2.1;

    // Odd::new_from_prob(0.5, 0.6) stores derived_from_no = 1 - 0.6 = 0.4
    assert!((over_diff - (0.5 - betano_over)).abs() < 1e-9);
    assert!((over_diff_from_no - (0.4 - betano_over)).abs() < 1e-9);
    // Odd::new_from_prob(0.5, 0.7) stores derived_from_no = 1 - 0.7 = 0.3
    assert!((under_diff - (0.5 - betano_under)).abs() < 1e-9);
    assert!((under_diff_from_no - (0.3 - betano_under)).abs() < 1e-9);
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
        vec![poly_total_market("poly-total", 2.5, 0.5, 0.6, 0.5, 0.7)],
    );

    let cluster = FixtureCluster::new(poly_game);

    assert!(cluster.live_statistics_diffs().is_empty());
}

#[test]
fn statistics_diffs_averages_accumulated_ticks() {
    let poly_game = game_with_markets(
        Platform::Polymarket,
        vec![poly_total_market("poly-total", 2.5, 0.5, 0.6, 0.5, 0.6)],
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

    let updated_poly = poly_total_market("poly-total", 2.5, 0.55, 0.55, 0.45, 0.45).1;
    cluster.update_markets(&poly_id, vec![updated_poly]);

    let tick2_over = 0.55 - 1.0 / 1.9;
    let tick2_over_from_no = 0.45 - 1.0 / 1.9;
    let tick2_under = 0.45 - 1.0 / 2.1;
    let tick2_under_from_no = 0.55 - 1.0 / 2.1;

    let means = cluster.statistics_diffs();

    assert_eq!(2, cluster.diffs[&total_type][&Outcome::Over].len());
    assert!(
        (means[&total_type][&Outcome::Over] - (tick1_over + tick2_over) / 2.0).abs()
            < 1e-9
    );
    assert!(
        (means[&total_type][&Outcome::Under] - (tick1_under + tick2_under) / 2.0).abs()
            < 1e-9
    );

    // live diff reflects the latest tick only, for both prob sources
    let live = cluster.live_statistics_diffs();
    let (live_over, live_over_from_no) = live[&total_type][&Outcome::Over];
    let (live_under, live_under_from_no) = live[&total_type][&Outcome::Under];
    assert!((live_over - tick2_over).abs() < 1e-9);
    assert!((live_over_from_no - tick2_over_from_no).abs() < 1e-9);
    assert!((live_under - tick2_under).abs() < 1e-9);
    assert!((live_under_from_no - tick2_under_from_no).abs() < 1e-9);
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
            vec![poly_moneyline_market("poly-moneyline", 0.55, 0.5, 0.45, 0.6)],
        ),
    ];

    let mut mean_diffs: HashMap<MarketType, HashMap<Outcome, f64>> = HashMap::new();
    mean_diffs
        .entry(MarketType::Moneyline)
        .or_default()
        .insert(Outcome::Home, 0.05);
    mean_diffs
        .entry(MarketType::Moneyline)
        .or_default()
        .insert(Outcome::Away, -0.03);

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

use chrono::{DateTime, TimeZone, Utc};

use super::*;

#[test]
fn close_marks_cluster_closed_and_updates_timestamp() {
    let mut cluster = FixtureCluster::new(game_with_markets(Platform::Betano, vec![]));

    let before = cluster.updated_at();
    std::thread::sleep(std::time::Duration::from_millis(2));
    cluster.close();

    assert!(cluster.is_closed());
    assert!(cluster.updated_at() > before);

    // closing twice must be a no-op for the timestamp
    let closed_at = cluster.updated_at();
    std::thread::sleep(std::time::Duration::from_millis(2));
    cluster.close();
    assert_eq!(closed_at, cluster.updated_at());
}

#[test]
fn display_lists_cluster_key_and_each_game_canonical_name() {
    let mut cluster = FixtureCluster::new(game_with_markets(Platform::Betano, vec![]));
    cluster
        .try_to_add_game(game_with_markets(Platform::Polymarket, vec![]))
        .unwrap();

    let display = cluster.to_string();

    assert!(display.contains("---------------"));
    assert!(display.contains(&cluster.key()));
    // key line + two canonical game lines each carry a "vs"
    assert_eq!(3, display.matches("vs").count());
}

#[test]
fn platform_games_filters_by_platform() {
    let mut cluster = FixtureCluster::new(game_with_markets(Platform::Betano, vec![]));
    cluster
        .try_to_add_game(game_with_markets(Platform::Polymarket, vec![]))
        .unwrap();

    let betano: Vec<&Game> = cluster.platform_games(&Platform::Betano).collect();
    let polymarket: Vec<&Game> = cluster.platform_games(&Platform::Polymarket).collect();
    let bwin: Vec<&Game> = cluster.platform_games(&Platform::Bwin).collect();

    assert_eq!(1, betano.len());
    assert_eq!(1, polymarket.len());
    assert!(bwin.is_empty());
}

#[test]
fn get_polymarket_impl_prob_returns_book_implied_probability() {
    let poly_game = Game::new_with_id(
        "poly-1",
        "FC Porto",
        "SL Benfica",
        "Portugal",
        "Liga Portugal",
        fixture_date(15, 30),
        Platform::Polymarket,
        vec![Market::Moneyline(MoneylineMarket::new(
            "poly-ml".to_string(),
            crate::domain::entities::Odd::new_from_prob(
                polymarket_client_sdk_v2::types::dec!(0.6),
                polymarket_client_sdk_v2::types::dec!(0.4),
            )
            .unwrap(),
            crate::domain::entities::Odd::new_from_prob(
                polymarket_client_sdk_v2::types::dec!(0.4),
                polymarket_client_sdk_v2::types::dec!(0.6),
            )
            .unwrap(),
        ))],
        None,
    );
    let mut cluster = FixtureCluster::new(game_with_markets(Platform::Betano, vec![]));
    cluster.try_to_add_game(poly_game).unwrap();

    let prob = cluster.get_polymarket_impl_prob_of_market_and_outcome(
        &MarketType::Moneyline,
        &crate::domain::entities::Outcome::Home,
    );

    assert!((prob.unwrap() - 0.6).abs() < 1e-9);
}

#[test]
fn get_polymarket_impl_prob_returns_none_without_polymarket_game() {
    let cluster = FixtureCluster::new(game_with_markets(Platform::Betano, vec![]));

    let prob = cluster.get_polymarket_impl_prob_of_market_and_outcome(
        &MarketType::Moneyline,
        &crate::domain::entities::Outcome::Home,
    );

    assert!(prob.is_none());
}

#[test]
fn from_persisted_restores_updated_at() {
    let updated_at = Utc.with_ymd_and_hms(2026, 5, 1, 12, 0, 0).unwrap();
    let cluster = FixtureCluster::from_persisted(
        "some-key".to_string(),
        vec![game_with_markets(Platform::Betano, vec![])],
        updated_at,
        Default::default(),
        false,
    );

    assert_eq!(updated_at, cluster.updated_at());
    assert_eq!("some-key", cluster.key());
    assert_eq!(1, cluster.game_count());
}

#[test]
fn key_is_the_representative_canonical_name() {
    let cluster = FixtureCluster::new(game_with_markets(Platform::Betano, vec![]));

    // "fc" is a weak token and is stripped from the normalized name
    assert!(cluster.key().starts_with("porto vs benfica @"));
}

#[test]
fn updated_at_is_stamped_on_creation() {
    let before = DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap();
    let cluster = FixtureCluster::new(game_with_markets(Platform::Betano, vec![]));

    assert!(cluster.updated_at() >= before);
}

#[test]
fn print_games_list_prints_one_line_per_game() {
    let mut cluster = FixtureCluster::new(game_with_markets(Platform::Betano, vec![]));
    cluster
        .try_to_add_game(game_with_markets(Platform::Polymarket, vec![]))
        .unwrap();

    // must not panic; output goes to stdout
    cluster.print_games_list();
}

#[test]
fn live_diffs_are_skipped_without_derived_from_no_probability() {
    // Odd::new does not carry the derived-from-no probability, so polymarket
    // samples without it must not produce live diffs
    let mut cluster = FixtureCluster::new(game_with_markets(
        Platform::Betano,
        vec![total_market("t-1", 2.5, 2.0, 2.0)],
    ));
    cluster
        .try_to_add_game(game_with_markets(
            Platform::Polymarket,
            vec![total_market("t-2", 2.5, 1.5, 2.0)],
        ))
        .unwrap();

    assert!(cluster.live_statistics_diffs().is_empty());
}

#[test]
fn live_diffs_median_averages_two_bookmakers() {
    use polymarket_client_sdk_v2::types::dec;

    use crate::domain::entities::Odd;

    // two bookmakers imply 0.5 and 0.6 -> median 0.55; polymarket implies 0.7
    let mut cluster = FixtureCluster::new(game_with_markets(
        Platform::Betano,
        vec![total_market("t-1", 2.5, 2.0, 2.0)],
    ));
    cluster
        .try_to_add_game(game_with_markets(
            Platform::Bwin,
            vec![total_market("t-3", 2.5, 5.0 / 3.0, 5.0 / 3.0)],
        ))
        .unwrap();
    cluster
        .try_to_add_game(game_with_markets(
            Platform::Polymarket,
            vec![(
                total_market_type(2.5),
                Market::Total(TotalMarket::new(
                    "t-2".to_string(),
                    Line(2.5),
                    Odd::new_from_prob(dec!(0.7), dec!(0.3)).unwrap(),
                    Odd::new_from_prob(dec!(0.3), dec!(0.7)).unwrap(),
                )),
            )],
        ))
        .unwrap();

    let diffs = cluster.live_statistics_diffs();

    let home = diffs
        .get(&MarketType::Total { line: 250 })
        .and_then(|inner| inner.get(&crate::domain::entities::Outcome::Over));
    let (diff, diff_from_no) = home.expect("expected a live diff for the Over outcome");
    // polymarket 0.7 - median(0.5, 0.6) = 0.15
    assert!((diff - 0.15).abs() < 1e-9);
    assert!((diff_from_no - 0.15).abs() < 1e-9);
}

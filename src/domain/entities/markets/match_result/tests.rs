use super::*;

#[test]
fn match_result_arbitrage_returns_none_for_empty_markets() {
    let result = MatchResultMarket::arbitrage_opportunites(&vec![]);

    assert_eq!(None, result);
}

#[test]
fn match_result_arbitrage_returns_none_when_implied_probabilities_sum_above_one() {
    let market = MatchResultMarket::new("single", Odd::new(2.0).unwrap(), Odd::new(3.0).unwrap(), Odd::new(4.0).unwrap());

    let result = MatchResultMarket::arbitrage_opportunites(&vec![market]);

    assert_eq!(None, result);
}

#[test]
fn match_result_arbitrage_selects_best_odd_per_outcome_across_bookmakers() {
    let first = MatchResultMarket::new(
        "book-a",
        Odd::new(2.5).unwrap(),
        Odd::new(3.0).unwrap(),
        Odd::new(3.0).unwrap(),
    );
    let second = MatchResultMarket::new(
        "book-b",
        Odd::new(2.0).unwrap(),
        Odd::new(3.5).unwrap(),
        Odd::new(3.0).unwrap(),
    );
    let third = MatchResultMarket::new(
        "book-c",
        Odd::new(2.0).unwrap(),
        Odd::new(3.0).unwrap(),
        Odd::new(3.6).unwrap(),
    );

    let result = MatchResultMarket::arbitrage_opportunites(&vec![first, second, third]);

    match result {
        Some(arb @ Arbitrage::MatchResultArbitrage(_)) => {
            // implied sum: 1/2.5 + 1/3.5 + 1/3.6 < 1 -> arbitrage
            assert!(arb.implied_probability_sum() < 1.0);
            let distribution = arb.stake_distribution(100.0).unwrap();
            assert_eq!(3, distribution.stakes.len());
            assert_eq!("book-a", distribution.stakes[0].market_id);
            assert_eq!("book-b", distribution.stakes[1].market_id);
            assert_eq!("book-c", distribution.stakes[2].market_id);
            assert!(distribution.guaranteed_profit > 0.0);
        }
        other => panic!("expected match result arbitrage, got {:?}", other),
    }
}

#[test]
fn match_result_arbitrage_rejects_market_with_only_two_sides_priced_below_threshold() {
    // odds that look attractive two-way but not three-way
    let market = MatchResultMarket::new(
        "single",
        Odd::new(3.4).unwrap(),
        Odd::new(1.2).unwrap(),
        Odd::new(3.4).unwrap(),
    );

    let result = MatchResultMarket::arbitrage_opportunites(&vec![market]);

    assert_eq!(None, result);
}

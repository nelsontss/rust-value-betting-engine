use super::*;

#[test]
fn moneyline_arbitrage_returns_none_for_empty_markets() {
    let result = MoneylineMarket::arbitrage_opportunites(&vec![]);

    assert_eq!(None, result);
}

#[test]
fn moneyline_arbitrage_returns_none_when_no_arbitrage_exists() {
    let market = MoneylineMarket::new(
        "single".to_string(),
        Odd::new(1.8).unwrap(),
        Odd::new(2.0).unwrap(),
    );

    let result = MoneylineMarket::arbitrage_opportunites(&vec![market]);

    assert_eq!(None, result);
}

#[test]
fn moneyline_arbitrage_combines_best_odds_from_different_bookmakers() {
    let first = MoneylineMarket::new(
        "betano".to_string(),
        Odd::new(2.2).unwrap(),
        Odd::new(1.8).unwrap(),
    );
    let second = MoneylineMarket::new(
        "bwin".to_string(),
        Odd::new(1.9).unwrap(),
        Odd::new(2.2).unwrap(),
    );

    let result = MoneylineMarket::arbitrage_opportunites(&vec![first, second]);

    match result {
        Some(Arbitrage::TwoWayArbitrage(arbitrage)) => {
            let distribution =
                Arbitrage::TwoWayArbitrage(arbitrage)
                    .stake_distribution(100.0)
                    .unwrap();
            assert_eq!(2, distribution.stakes.len());
            // best home odd (2.2) must come from betano, best away odd (2.2) from bwin
            assert_eq!("betano", distribution.stakes[0].market_id);
            assert_eq!("bwin", distribution.stakes[1].market_id);
            assert!(distribution.guaranteed_profit > 0.0);
        }
        other => panic!("expected two way arbitrage, got {:?}", other),
    }
}

#[test]
fn moneyline_id_returns_stored_id() {
    let market = MoneylineMarket::new(
        "ml-id".to_string(),
        Odd::new(2.0).unwrap(),
        Odd::new(2.0).unwrap(),
    );

    assert_eq!("ml-id", market.id());
}

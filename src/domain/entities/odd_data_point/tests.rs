use super::*;
use crate::domain::entities::markets::match_result::MatchResultMarket;

#[test]
fn new_with_datetime_stores_market_and_timestamp() {
    let market = Market::match_result("mr", 2.0, 3.0, 4.0).unwrap();
    let datetime = DateTime::from_timestamp(1_750_000_000, 0).unwrap();

    let point = MarketDataPoint::new_with_datetime(market.clone(), datetime);

    assert_eq!(&market, point.market());
    assert_eq!(&datetime, point.datetime());
}

#[test]
fn new_stamps_current_time() {
    let market = Market::moneyline("ml", 2.0, 1.8).unwrap();
    let before = Utc::now();

    let point = MarketDataPoint::new(market);

    let after = Utc::now();
    assert!(*point.datetime() >= before && *point.datetime() <= after);
}

#[test]
fn data_point_keeps_market_variant_intact() {
    let market = Market::MatchResult(MatchResultMarket::new(
        "mr-1",
        crate::domain::entities::Odd::new(2.0).unwrap(),
        crate::domain::entities::Odd::new(3.0).unwrap(),
        crate::domain::entities::Odd::new(4.0).unwrap(),
    ));

    let point = MarketDataPoint::new_with_datetime(market, DateTime::from_timestamp(0, 0).unwrap());

    assert!(matches!(point.market(), Market::MatchResult(_)));
}

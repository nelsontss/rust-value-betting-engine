use polymarket_client_sdk_v2::types::dec;

use super::trade_config::TradeConfig;

#[test]
fn trade_config_keeps_price_band_ordered_within_probabilities() {
    assert!(TradeConfig::MIN_PRICE > dec!(0));
    assert!(TradeConfig::MIN_PRICE < TradeConfig::MAX_PRICE);
    assert!(TradeConfig::MAX_PRICE < dec!(1));
}

#[test]
fn trade_config_offsets_are_symmetric_and_positive() {
    assert!(TradeConfig::BUY_OFFSET > chrono::Duration::zero());
    assert_eq!(TradeConfig::BUY_OFFSET, TradeConfig::SELL_OFFSET);
}

#[test]
fn trade_config_bankroll_and_volume_are_positive() {
    assert!(TradeConfig::BANKROLL > dec!(0));
    assert!(TradeConfig::MAX_VOLUME > dec!(0));
}

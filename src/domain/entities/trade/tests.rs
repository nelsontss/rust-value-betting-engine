use super::*;
use polymarket_client_sdk_v2::types::dec;

fn open_trade() -> Trade {
    Trade::open_trade(
        Some("order-1".to_string()),
        "market-1".to_string(),
        "token-1".to_string(),
        Side::Buy,
        dec!(10.0),
        dec!(0.45),
        1_000,
        true,
        TradeStrategy::DrawTimeDecay,
    )
}

#[test]
fn trade_status_serializes_to_lowercase_strings() {
    assert_eq!("open", TradeStatus::Open.to_string());
    assert_eq!("closed", TradeStatus::Closed.to_string());
}

#[test]
fn trade_status_parses_from_lowercase_strings() {
    assert_eq!(Ok(TradeStatus::Open), "open".parse::<TradeStatus>());
    assert_eq!(Ok(TradeStatus::Closed), "closed".parse::<TradeStatus>());
    assert!("cancelled".parse::<TradeStatus>().is_err());
}

#[test]
fn trade_strategy_serializes_to_snake_case_strings() {
    assert_eq!("draw_time_decay", TradeStrategy::DrawTimeDecay.to_string());
}

#[test]
fn trade_strategy_parses_from_snake_case_strings() {
    assert_eq!(
        Ok(TradeStrategy::DrawTimeDecay),
        "draw_time_decay".parse::<TradeStrategy>()
    );
    assert!("martingale".parse::<TradeStrategy>().is_err());
}

#[test]
fn open_trade_initializes_an_open_paper_trade() {
    let trade = open_trade();

    assert_eq!("market-1", trade.market_id);
    assert_eq!("token-1", trade.token_id);
    assert_eq!(dec!(10.0), trade.size);
    assert_eq!(dec!(0.45), trade.entry_price);
    assert_eq!(1_000, trade.entry_time);
    assert_eq!(TradeStatus::Open, trade.status);
    assert!(trade.paper);
    assert_eq!(Some("order-1".to_string()), trade.buy_order_id);
    assert_eq!(None, trade.sell_order_id);
    assert_eq!(None, trade.exit_price);
    assert_eq!(None, trade.exit_time);
    assert_eq!(None, trade.pnl);
    assert_eq!(trade.created_at, trade.updated_at);
    assert!(!trade.id.is_empty());
}

#[test]
fn open_trade_generates_unique_ids() {
    let first = open_trade();
    let second = open_trade();

    assert_ne!(first.id, second.id);
}

#[test]
fn close_trade_records_exit_and_computes_pnl() {
    let mut trade = open_trade();

    trade.close_trade(dec!(0.60), 2_000, Some("sell-order-1".to_string()));

    assert_eq!(TradeStatus::Closed, trade.status);
    assert_eq!(Some(dec!(0.60)), trade.exit_price);
    assert_eq!(Some(2_000), trade.exit_time);
    assert_eq!(Some(dec!(1.5)), trade.pnl);
    assert_eq!(Some("sell-order-1".to_string()), trade.sell_order_id);
    assert!(trade.updated_at >= trade.created_at);
}

#[test]
fn close_trade_computes_negative_pnl_for_losing_trade() {
    let mut trade = open_trade();

    trade.close_trade(dec!(0.30), 2_000, None);

    assert_eq!(Some(dec!(-1.5)), trade.pnl);
    assert_eq!(None, trade.sell_order_id);
}

#[test]
fn side_str_maps_buy_and_sell() {
    let mut trade = open_trade();
    assert_eq!("buy", trade.side_str());

    trade.side = Side::Sell;
    assert_eq!("sell", trade.side_str());
}

#[test]
fn side_from_str_round_trips_known_sides() {
    assert!(matches!(side_from_str("buy"), Ok(Side::Buy)));
    assert!(matches!(side_from_str("sell"), Ok(Side::Sell)));
    assert!(side_from_str("hold").is_err());
}

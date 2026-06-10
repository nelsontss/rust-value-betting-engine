use crate::domain::{
    Arbitrage,
    entities::{Odd, TwoWayArbitrage, odd::best_odd_with_id},
};

#[derive(Debug, Clone, PartialEq)]
pub struct DoubleChanceMarket {
    id: String,
    pub home_or_draw: Odd,
    pub home_or_away: Odd,
    pub draw_or_away: Odd,
}

impl DoubleChanceMarket {
    pub fn new(id: String, home_or_draw: Odd, home_or_away: Odd, draw_or_away: Odd) -> Self {
        Self {
            id,
            home_or_draw,
            home_or_away,
            draw_or_away,
        }
    }

    pub fn arbitrage_opportunites(markets: &Vec<DoubleChanceMarket>) -> Option<Arbitrage> {
        if markets.is_empty() {
            return None;
        }

        let best_home_or_draw = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.home_or_draw, market.id.clone())),
        );
        let best_home_or_away = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.home_or_away, market.id.clone())),
        );
        let best_draw_or_away = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.draw_or_away, market.id.clone())),
        );

        let sums = [
            (1.0 / best_home_or_draw.0.get()) + (1.0 / best_home_or_away.0.get()),
            (1.0 / best_home_or_draw.0.get()) + (1.0 / best_draw_or_away.0.get()),
            (1.0 / best_home_or_away.0.get()) + (1.0 / best_draw_or_away.0.get()),
        ];

        let mut best_idx = None;
        let mut best_roi = 0.0;
        for (i, &sum) in sums.iter().enumerate() {
            if sum < 1.0 {
                let roi = (1.0 / sum) - 1.0;
                if roi > best_roi {
                    best_roi = roi;
                    best_idx = Some(i);
                }
            }
        }

        match best_idx? {
            0 => Some(Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
                best_home_or_draw, best_home_or_away, sums[0],
            ))),
            1 => Some(Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
                best_home_or_draw, best_draw_or_away, sums[1],
            ))),
            _ => Some(Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
                best_home_or_away, best_draw_or_away, sums[2],
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_chance_arbitrage_best_pair_found() {
        let market = DoubleChanceMarket::new(
            "single".to_string(),
            Odd::new(2.2).unwrap(),
            Odd::new(2.2).unwrap(),
            Odd::new(2.2).unwrap(),
        );

        let result = DoubleChanceMarket::arbitrage_opportunites(&vec![market]);

        assert!(matches!(result, Some(Arbitrage::TwoWayArbitrage(_))));
    }

    #[test]
    fn double_chance_arbitrage_picks_best_pair() {
        let first = DoubleChanceMarket::new(
            "first".to_string(),
            Odd::new(2.0).unwrap(),
            Odd::new(3.0).unwrap(),
            Odd::new(100.0).unwrap(),
        );
        let second = DoubleChanceMarket::new(
            "second".to_string(),
            Odd::new(2.4).unwrap(),
            Odd::new(2.0).unwrap(),
            Odd::new(2.0).unwrap(),
        );

        let result = DoubleChanceMarket::arbitrage_opportunites(&vec![first, second]).unwrap();

        // best pair should be 1X+12: 1/2.4 + 1/3 = 0.75 -> roi = 33.3%
        // 1X+X2: 1/2.4 + 1/100 = 0.427 -> roi = 134%
        // 12+X2: 1/3 + 1/100 = 0.343 -> roi = 191%
        // So 12+X2 has best roi at 191%
        assert!((result.roi() - 1.913).abs() < 0.01);
    }

    #[test]
    fn double_chance_arbitrage_returns_none_when_no_arb() {
        let market = DoubleChanceMarket::new(
            "single".to_string(),
            Odd::new(1.3).unwrap(),
            Odd::new(1.4).unwrap(),
            Odd::new(1.5).unwrap(),
        );

        let result = DoubleChanceMarket::arbitrage_opportunites(&vec![market]);

        assert_eq!(None, result);
    }

    #[test]
    fn double_chance_arbitrage_returns_none_for_empty_markets() {
        let result = DoubleChanceMarket::arbitrage_opportunites(&vec![]);

        assert_eq!(None, result);
    }
}

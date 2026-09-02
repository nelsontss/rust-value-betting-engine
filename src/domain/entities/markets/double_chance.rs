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
    pub fn id(&self) -> String {
        self.id.clone()
    }

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
                best_home_or_draw,
                best_home_or_away,
                sums[0],
            ))),
            1 => Some(Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
                best_home_or_draw,
                best_draw_or_away,
                sums[1],
            ))),
            _ => Some(Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
                best_home_or_away,
                best_draw_or_away,
                sums[2],
            ))),
        }
    }
}

#[cfg(test)]
mod tests;

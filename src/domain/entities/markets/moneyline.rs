use crate::domain::{
    Arbitrage,
    entities::{Odd, TwoWayArbitrage, odd::best_odd_with_id},
};

#[derive(Debug, Clone, PartialEq)]
pub struct MoneylineMarket {
    id: String,
    pub home: Odd,
    pub away: Odd,
}

impl MoneylineMarket {
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn new(id: String, home: Odd, away: Odd) -> Self {
        Self { id, home, away }
    }

    pub fn arbitrage_opportunites(markets: &Vec<MoneylineMarket>) -> Option<Arbitrage> {
        if markets.is_empty() {
            return None;
        }

        let best_first = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.home, market.id.clone())),
        );
        let best_second = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.away, market.id.clone())),
        );

        let implied_probability_sum = (1.0 / best_first.0.get()) + (1.0 / best_second.0.get());

        if implied_probability_sum < 1.0 {
            Some(Arbitrage::TwoWayArbitrage(TwoWayArbitrage::new(
                best_first,
                best_second,
                implied_probability_sum,
            )))
        } else {
            None
        }
    }
}

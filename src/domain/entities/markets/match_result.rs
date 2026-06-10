use crate::domain::{
    Arbitrage,
    entities::{MatchResultArbitrage, Odd, odd::best_odd_with_id},
};

#[derive(Debug, Clone, PartialEq)]
pub struct MatchResultMarket {
    pub id: String,
    pub home: Odd,
    pub draw: Odd,
    pub away: Odd,
}

impl MatchResultMarket {
    pub fn new(id: &str, home: Odd, draw: Odd, away: Odd) -> Self {
        Self {
            id: id.to_string(),
            home,
            draw,
            away,
        }
    }

    pub fn arbitrage_opportunites(markets: &Vec<MatchResultMarket>) -> Option<Arbitrage> {
        if markets.is_empty() {
            return None;
        }

        let best_home = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.home, market.id.clone())),
        );

        let best_draw = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.draw, market.id.clone())),
        );

        let best_away = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.away, market.id.clone())),
        );

        let implied_probability_sum =
            (1.0 / best_home.0.get()) + (1.0 / best_draw.0.get()) + (1.0 / best_away.0.get());

        if implied_probability_sum < 1.0 {
            Some(Arbitrage::MatchResultArbitrage(MatchResultArbitrage::new(
                best_home,
                best_draw,
                best_away,
                implied_probability_sum,
            )))
        } else {
            None
        }
    }
}

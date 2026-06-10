use crate::domain::{
    Arbitrage,
    entities::{Odd, ThreeWayLineArbitrage, markets::Line, odd::best_odd_with_id},
};

#[derive(Debug, Clone, PartialEq)]
pub struct HandicapMarket {
    id: String,
    pub line: Line,
    pub home: Odd,
    pub draw: Odd,
    pub away: Odd,
}

impl HandicapMarket {
    pub fn new(id: &str, line: Line, home: Odd, draw: Odd, away: Odd) -> Self {
        Self {
            id: id.to_string(),
            line,
            home,
            draw,
            away,
        }
    }

    pub fn arbitrage_opportunites(markets: &Vec<HandicapMarket>) -> Option<Arbitrage> {
        let line = markets.first()?.line;

        if markets.iter().any(|market| market.line != line) {
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
            Some(Arbitrage::ThreeWayLineArbitrage(
                ThreeWayLineArbitrage::new(
                    line,
                    best_home,
                    best_draw,
                    best_away,
                    implied_probability_sum,
                ),
            ))
        } else {
            None
        }
    }
}

use crate::domain::{
    Arbitrage,
    entities::{
        Odd, TwoWayLineArbitrage,
        markets::{Line, ceil_int, floor_int, guaranteed_profit, line_components},
        odd::best_odd_with_id,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct AsianHandicapMarket {
    id: String,
    pub line: Line,
    pub home: Odd,
    pub away: Odd,
}

impl AsianHandicapMarket {
    pub fn new(id: String, line: Line, home: Odd, away: Odd) -> Self {
        Self {
            id,
            line,
            home,
            away,
        }
    }

    pub fn arbitrage_opportunites(markets: &Vec<AsianHandicapMarket>) -> Option<Arbitrage> {
        let line = markets.first()?.line;

        if markets.iter().any(|market| market.line.key() != line.key()) {
            return None;
        }

        let best_home = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.home, market.id.clone())),
        );
        let best_away = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.away, market.id.clone())),
        );

        let implied_probability_sum = (1.0 / best_home.0.get()) + (1.0 / best_away.0.get());
        let guaranteed_profit =
            guaranteed_profit(&asian_handicap_scenarios(line, best_home.0, best_away.0));

        if guaranteed_profit > 0.0 {
            Some(Arbitrage::TwoWayLineArbitrage(TwoWayLineArbitrage::new(
                line,
                best_home,
                best_away,
                implied_probability_sum,
            )))
        } else {
            None
        }
    }
}

fn asian_handicap_scenarios(line: Line, home: Odd, away: Odd) -> Vec<(f64, f64)> {
    let components = line_components(line);
    let thresholds: Vec<i32> = components.iter().map(|component| -component).collect();
    let min_margin = thresholds
        .iter()
        .map(|threshold| floor_int(*threshold))
        .min()
        .unwrap()
        - 1;
    let max_margin = thresholds
        .iter()
        .map(|threshold| ceil_int(*threshold))
        .max()
        .unwrap()
        + 1;

    (min_margin..=max_margin)
        .map(|margin| {
            let home_multiplier = components
                .iter()
                .map(|component| home_return_multiplier(*component, margin, home))
                .sum::<f64>()
                / components.len() as f64;
            let away_multiplier = components
                .iter()
                .map(|component| away_return_multiplier(*component, margin, away))
                .sum::<f64>()
                / components.len() as f64;

            (home_multiplier, away_multiplier)
        })
        .collect()
}

fn home_return_multiplier(component: i32, margin: i32, odd: Odd) -> f64 {
    let adjusted_margin = margin * 100 + component;

    if adjusted_margin > 0 {
        odd.get()
    } else if adjusted_margin == 0 {
        1.0
    } else {
        0.0
    }
}

fn away_return_multiplier(component: i32, margin: i32, odd: Odd) -> f64 {
    let adjusted_margin = margin * 100 + component;

    if adjusted_margin < 0 {
        odd.get()
    } else if adjusted_margin == 0 {
        1.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asian_handicap_market_arbitrage_opportunites_handles_quarter_lines() {
        let first_market = AsianHandicapMarket::new(
            "first-asian".to_string(),
            Line(-0.25),
            Odd::new(2.2).unwrap(),
            Odd::new(1.8).unwrap(),
        );
        let second_market = AsianHandicapMarket::new(
            "second-asian".to_string(),
            Line(-0.25),
            Odd::new(1.8).unwrap(),
            Odd::new(2.2).unwrap(),
        );

        let result =
            AsianHandicapMarket::arbitrage_opportunites(&vec![first_market, second_market]);

        assert!(matches!(result, Some(Arbitrage::TwoWayLineArbitrage(_))));
    }
}

use crate::domain::{
    Arbitrage,
    entities::{
        Odd, TwoWayLineArbitrage,
        markets::{Line, ceil_int, floor_int, guaranteed_profit, line_components},
        odd::best_odd_with_id,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct TotalMarket {
    id: String,
    pub line: Line,
    pub over: Odd,
    pub under: Odd,
}

impl TotalMarket {
    pub fn new(id: String, line: Line, over: Odd, under: Odd) -> Self {
        Self {
            id,
            line,
            over,
            under,
        }
    }

    pub fn arbitrage_opportunites(markets: &Vec<TotalMarket>) -> Option<Arbitrage> {
        let line = markets.first()?.line;

        if markets.iter().any(|market| market.line.key() != line.key()) {
            return None;
        }

        let best_home = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.over, market.id.clone())),
        );
        let best_away = best_odd_with_id(
            markets
                .iter()
                .map(|market| (market.under, market.id.clone())),
        );

        let implied_probability_sum = (1.0 / best_home.0.get()) + (1.0 / best_away.0.get());
        let guaranteed_profit =
            guaranteed_profit(&total_market_scenarios(line, best_home.0, best_away.0));

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

fn total_market_scenarios(line: Line, over: Odd, under: Odd) -> Vec<(f64, f64)> {
    let components = line_components(line);
    let min_total = components
        .iter()
        .map(|component| floor_int(*component))
        .min()
        .unwrap()
        - 1;
    let max_total = components
        .iter()
        .map(|component| ceil_int(*component))
        .max()
        .unwrap()
        + 1;

    (min_total..=max_total)
        .map(|total| {
            let over_multiplier = components
                .iter()
                .map(|component| over_return_multiplier(*component, total, over))
                .sum::<f64>()
                / components.len() as f64;
            let under_multiplier = components
                .iter()
                .map(|component| under_return_multiplier(*component, total, under))
                .sum::<f64>()
                / components.len() as f64;

            (over_multiplier, under_multiplier)
        })
        .collect()
}

fn over_return_multiplier(component: i32, total: i32, odd: Odd) -> f64 {
    let total_key = total * 100;

    if total_key > component {
        odd.get()
    } else if total_key == component {
        1.0
    } else {
        0.0
    }
}

fn under_return_multiplier(component: i32, total: i32, odd: Odd) -> f64 {
    let total_key = total * 100;

    if total_key < component {
        odd.get()
    } else if total_key == component {
        1.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_market_arbitrage_opportunites_rejects_integer_lines_with_push_state() {
        let first_market = TotalMarket::new(
            "first-total".to_string(),
            Line(2.0),
            Odd::new(2.2).unwrap(),
            Odd::new(1.8).unwrap(),
        );
        let second_market = TotalMarket::new(
            "second-total".to_string(),
            Line(2.0),
            Odd::new(1.8).unwrap(),
            Odd::new(2.2).unwrap(),
        );

        let result = TotalMarket::arbitrage_opportunites(&vec![first_market, second_market]);

        assert_eq!(None, result);
    }
}

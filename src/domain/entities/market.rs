use super::arbitrage::{
    Arbitrage, MatchResultArbitrage, ThreeWayLineArbitrage, TwoWayArbitrage, TwoWayLineArbitrage,
};
use std::cmp::Ordering;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Odd(f64);

impl Odd {
    pub fn new(value: f64) -> Result<Self, OddError> {
        if !value.is_finite() || value <= 0.0 {
            return Err(OddError::NonPositive(value));
        }

        Ok(Self(value))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OddError {
    NonPositive(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line(pub f32);

impl Line {
    pub fn key(self) -> i32 {
        (self.0 * 100.0).round() as i32
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchResultMarket {
    id: String,
    home: Odd,
    draw: Odd,
    away: Odd,
}

fn best_odd_with_id<I>(markets: I) -> (Odd, String)
where
    I: IntoIterator<Item = (Odd, String)>,
{
    markets
        .into_iter()
        .max_by(|left, right| {
            left.0
                .get()
                .partial_cmp(&right.0.get())
                .unwrap_or(Ordering::Equal)
        })
        .expect("markets must be non-empty")
}

fn line_components(line: Line) -> Vec<i32> {
    let key = line.key();
    let fractional = key.abs() % 100;

    if fractional == 25 || fractional == 75 {
        let direction = if key.is_negative() { -25 } else { 25 };
        vec![key - direction, key + direction]
    } else {
        vec![key]
    }
}

fn floor_int(key: i32) -> i32 {
    key.div_euclid(100)
}

fn ceil_int(key: i32) -> i32 {
    if key.rem_euclid(100) == 0 {
        key.div_euclid(100)
    } else {
        key.div_euclid(100) + 1
    }
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

fn guaranteed_profit(scenarios: &[(f64, f64)]) -> f64 {
    let mut candidate_splits = vec![0.0, 1.0];

    for (index, left) in scenarios.iter().enumerate() {
        let left_slope = left.0 - left.1;
        let left_intercept = left.1 - 1.0;

        for right in scenarios.iter().skip(index + 1) {
            let right_slope = right.0 - right.1;
            let right_intercept = right.1 - 1.0;

            if (left_slope - right_slope).abs() < f64::EPSILON {
                continue;
            }

            let split = (right_intercept - left_intercept) / (left_slope - right_slope);

            if (0.0..=1.0).contains(&split) {
                candidate_splits.push(split);
            }
        }
    }

    candidate_splits
        .into_iter()
        .map(|split| {
            scenarios
                .iter()
                .map(|(first_multiplier, second_multiplier)| {
                    split * first_multiplier + (1.0 - split) * second_multiplier - 1.0
                })
                .fold(f64::INFINITY, f64::min)
        })
        .fold(f64::NEG_INFINITY, f64::max)
}

impl MatchResultMarket {
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

#[derive(Debug, Clone, PartialEq)]
pub struct MoneylineMarket {
    id: String,
    pub home: Odd,
    pub away: Odd,
}

impl MoneylineMarket {
    pub fn new(id: &str, home: Odd, away: Odd) -> Self {
        Self {
            id: id.to_string(),
            home,
            away,
        }
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

#[derive(Debug, Clone, PartialEq)]
pub struct TotalMarket {
    id: String,
    pub line: Line,
    pub over: Odd,
    pub under: Odd,
}

impl TotalMarket {
    pub fn new(id: &str, line: Line, over: Odd, under: Odd) -> Self {
        Self {
            id: id.to_string(),
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

#[derive(Debug, Clone, PartialEq)]
pub struct HandicapMarket {
    id: String,
    pub line: Line,
    pub home: Odd,
    pub draw: Odd,
    pub away: Odd,
}

impl HandicapMarket {
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

#[derive(Debug, Clone, PartialEq)]
pub struct AsianHandicapMarket {
    id: String,
    pub line: Line,
    pub home: Odd,
    pub away: Odd,
}

impl AsianHandicapMarket {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Market {
    MatchResult(MatchResultMarket),
    Moneyline(MoneylineMarket),
    Total(TotalMarket),
    Handicap(HandicapMarket),
    AsianHandicap(AsianHandicapMarket),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarketType {
    MatchResult,
    Moneyline,
    Total { line: i32 },
    Handicap { line: i32 },
    AsianHandicap { line: i32 },
}

impl MarketType {
    pub fn from(market: &Market) -> MarketType {
        match market {
            Market::MatchResult(_) => MarketType::MatchResult,
            Market::Moneyline(_) => MarketType::Moneyline,
            Market::Total(market) => MarketType::Total {
                line: market.line.key(),
            },
            Market::AsianHandicap(market) => MarketType::AsianHandicap {
                line: market.line.key(),
            },
            Market::Handicap(market) => MarketType::Handicap {
                line: market.line.key(),
            },
        }
    }
}

pub enum MarketGroup {
    MatchResult(Vec<MatchResultMarket>),
    Moneyline(Vec<MoneylineMarket>),
    Total {
        line: i32,
        markets: Vec<TotalMarket>,
    },
    Handicap {
        line: i32,
        markets: Vec<HandicapMarket>,
    },
    AsianHandicap {
        line: i32,
        markets: Vec<AsianHandicapMarket>,
    },
}

impl MarketGroup {
    pub fn from_market(market: Market) -> Self {
        match market {
            Market::MatchResult(market) => Self::MatchResult(vec![market]),
            Market::Moneyline(market) => Self::Moneyline(vec![market]),
            Market::Total(market) => Self::Total {
                line: market.line.key(),
                markets: vec![market],
            },
            Market::Handicap(market) => Self::Handicap {
                line: market.line.key(),
                markets: vec![market],
            },
            Market::AsianHandicap(market) => Self::AsianHandicap {
                line: market.line.key(),
                markets: vec![market],
            },
        }
    }

    pub fn market_type(&self) -> MarketType {
        match self {
            MarketGroup::MatchResult(_) => MarketType::MatchResult,
            MarketGroup::Moneyline(_) => MarketType::Moneyline,
            MarketGroup::Total { line, .. } => MarketType::Total { line: *line },
            MarketGroup::Handicap { line, .. } => MarketType::Handicap { line: *line },
            MarketGroup::AsianHandicap { line, .. } => MarketType::AsianHandicap { line: *line },
        }
    }

    pub fn push_market(&mut self, market: Market) -> Result<(), MarketGroupError> {
        match (self, market) {
            (MarketGroup::MatchResult(markets), Market::MatchResult(market)) => {
                markets.push(market)
            }
            (MarketGroup::Moneyline(markets), Market::Moneyline(market)) => markets.push(market),
            (MarketGroup::Total { line, markets }, Market::Total(market))
                if *line == market.line.key() =>
            {
                markets.push(market)
            }
            (MarketGroup::Handicap { line, markets }, Market::Handicap(market))
                if *line == market.line.key() =>
            {
                markets.push(market)
            }
            (MarketGroup::AsianHandicap { line, markets }, Market::AsianHandicap(market))
                if *line == market.line.key() =>
            {
                markets.push(market)
            }
            _ => return Err(MarketGroupError::MarketTypeAndGroupDontMatch),
        }

        Ok(())
    }

    pub fn arbitrage(&self) -> Option<Arbitrage> {
        match self {
            MarketGroup::MatchResult(markets) => MatchResultMarket::arbitrage_opportunites(markets),
            MarketGroup::Moneyline(markets) => MoneylineMarket::arbitrage_opportunites(markets),
            MarketGroup::Total { markets, .. } => TotalMarket::arbitrage_opportunites(markets),
            MarketGroup::Handicap { markets, .. } => {
                HandicapMarket::arbitrage_opportunites(markets)
            }
            MarketGroup::AsianHandicap { markets, .. } => {
                AsianHandicapMarket::arbitrage_opportunites(markets)
            }
        }
    }
}

pub enum MarketGroupError {
    MarketTypeAndGroupDontMatch,
}

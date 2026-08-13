use serde::Serialize;

use crate::domain::{
    OddError,
    entities::{
        Odd,
        markets::{
            Line, asian_handicap::AsianHandicapMarket, double_chance::DoubleChanceMarket,
            handicap::HandicapMarket, match_result::MatchResultMarket, moneyline::MoneylineMarket,
            total::TotalMarket,
        },
    },
};

use super::arbitrage::Arbitrage;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub enum Market {
    MatchResult(MatchResultMarket),
    Moneyline(MoneylineMarket),
    DoubleChance(DoubleChanceMarket),
    Total(TotalMarket),
    Handicap(HandicapMarket),
    AsianHandicap(AsianHandicapMarket),
}

impl Market {
    pub fn match_result(id: &str, home: f64, draw: f64, away: f64) -> Result<Self, OddError> {
        Ok(Self::MatchResult(MatchResultMarket::new(
            id,
            Odd::new(home)?,
            Odd::new(draw)?,
            Odd::new(away)?,
        )))
    }

    pub fn moneyline(id: &str, home: f64, away: f64) -> Result<Self, OddError> {
        Ok(Self::Moneyline(MoneylineMarket::new(
            id.to_string(),
            Odd::new(home)?,
            Odd::new(away)?,
        )))
    }

    pub fn double_chance(
        id: &str,
        home_or_draw: f64,
        home_or_away: f64,
        draw_or_away: f64,
    ) -> Result<Self, OddError> {
        Ok(Self::DoubleChance(DoubleChanceMarket::new(
            id.to_string(),
            Odd::new(home_or_draw)?,
            Odd::new(home_or_away)?,
            Odd::new(draw_or_away)?,
        )))
    }

    pub fn total(id: &str, line: f32, over: f64, under: f64) -> Result<Self, OddError> {
        Ok(Self::Total(TotalMarket::new(
            id.to_string(),
            Line(line),
            Odd::new(over)?,
            Odd::new(under)?,
        )))
    }

    pub fn handicap(
        id: &str,
        line: f32,
        home: f64,
        draw: f64,
        away: f64,
    ) -> Result<Self, OddError> {
        Ok(Self::Handicap(HandicapMarket::new(
            id,
            Line(line),
            Odd::new(home)?,
            Odd::new(draw)?,
            Odd::new(away)?,
        )))
    }

    pub fn asian_handicap(id: &str, line: f32, home: f64, away: f64) -> Result<Self, OddError> {
        Ok(Self::AsianHandicap(AsianHandicapMarket::new(
            id.to_string(),
            Line(line),
            Odd::new(home)?,
            Odd::new(away)?,
        )))
    }

    pub fn odd_for_outcome(&self, outcome: &Outcome) -> Option<Odd> {
        match self {
            Market::MatchResult(m) => match outcome {
                Outcome::Home => Some(m.home),
                Outcome::Draw => Some(m.draw),
                Outcome::Away => Some(m.away),
                _ => None,
            },
            Market::Moneyline(m) => match outcome {
                Outcome::Home => Some(m.home),
                Outcome::Away => Some(m.away),
                _ => None,
            },
            Market::DoubleChance(m) => match outcome {
                Outcome::HomeOrDraw => Some(m.home_or_draw),
                Outcome::HomeOrAway => Some(m.home_or_away),
                Outcome::DrawOrAway => Some(m.draw_or_away),
                _ => None,
            },
            Market::Total(m) => match outcome {
                Outcome::Over => Some(m.over),
                Outcome::Under => Some(m.under),
                _ => None,
            },
            Market::Handicap(m) => match outcome {
                Outcome::Home => Some(m.home),
                Outcome::Draw => Some(m.draw),
                Outcome::Away => Some(m.away),
                _ => None,
            },
            Market::AsianHandicap(m) => match outcome {
                Outcome::Home => Some(m.home),
                Outcome::Away => Some(m.away),
                _ => None,
            },
        }
    }

    pub fn sum_implied_probabilities(&self) -> f64 {
        let market_type = MarketType::from(self);

        market_type
            .outcomes()
            .iter()
            .map(|outcome| {
                self.odd_for_outcome(outcome)
                    .expect("Outcome not in market")
                    .get_implied_probability()
            })
            .sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Outcome {
    Home,
    Draw,
    Away,
    Over,
    Under,
    HomeOrDraw,
    HomeOrAway,
    DrawOrAway,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum MarketType {
    MatchResult,
    Moneyline,
    DoubleChance,
    Total { line: i32 },
    Handicap { line: i32 },
    AsianHandicap { line: i32 },
}

impl MarketType {
    pub fn variant_name(&self) -> &str {
        match self {
            MarketType::MatchResult => "MatchResult",
            MarketType::Moneyline => "Moneyline",
            MarketType::DoubleChance => "DoubleChance",
            MarketType::Total { .. } => "Total",
            MarketType::Handicap { .. } => "Handicap",
            MarketType::AsianHandicap { .. } => "AsianHandicap",
        }
    }

    pub fn to_key_string(&self) -> String {
        match self {
            MarketType::MatchResult => "MatchResult".to_string(),
            MarketType::Moneyline => "Moneyline".to_string(),
            MarketType::DoubleChance => "DoubleChance".to_string(),
            MarketType::Total { line } => format!("Total:{}", line),
            MarketType::Handicap { line } => format!("Handicap:{}", line),
            MarketType::AsianHandicap { line } => format!("AsianHandicap:{}", line),
        }
    }

    pub fn from(market: &Market) -> MarketType {
        match market {
            Market::MatchResult(_) => MarketType::MatchResult,
            Market::Moneyline(_) => MarketType::Moneyline,
            Market::DoubleChance(_) => MarketType::DoubleChance,
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

    pub fn outcomes(&self) -> Vec<Outcome> {
        match self {
            MarketType::MatchResult => vec![Outcome::Home, Outcome::Draw, Outcome::Away],
            MarketType::Moneyline => vec![Outcome::Home, Outcome::Away],
            MarketType::DoubleChance => vec![
                Outcome::HomeOrDraw,
                Outcome::HomeOrAway,
                Outcome::DrawOrAway,
            ],
            MarketType::Total { .. } => vec![Outcome::Over, Outcome::Under],
            MarketType::Handicap { .. } => vec![Outcome::Home, Outcome::Draw, Outcome::Away],
            MarketType::AsianHandicap { .. } => vec![Outcome::Home, Outcome::Away],
        }
    }
}

pub enum MarketGroup {
    MatchResult(Vec<MatchResultMarket>),
    Moneyline(Vec<MoneylineMarket>),
    DoubleChance(Vec<DoubleChanceMarket>),
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
            Market::DoubleChance(market) => Self::DoubleChance(vec![market]),
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
            MarketGroup::DoubleChance(_) => MarketType::DoubleChance,
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
            (MarketGroup::DoubleChance(markets), Market::DoubleChance(market)) => {
                markets.push(market)
            }
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
            MarketGroup::DoubleChance(markets) => {
                DoubleChanceMarket::arbitrage_opportunites(markets)
            }
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

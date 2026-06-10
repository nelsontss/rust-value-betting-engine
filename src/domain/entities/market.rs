use crate::domain::{
    OddError,
    entities::{
        Odd,
        markets::{
            Line, asian_handicap::AsianHandicapMarket, double_chance::DoubleChanceMarket,
            handicap::HandicapMarket, match_result::MatchResultMarket,
            moneyline::MoneylineMarket, total::TotalMarket,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarketType {
    MatchResult,
    Moneyline,
    DoubleChance,
    Total { line: i32 },
    Handicap { line: i32 },
    AsianHandicap { line: i32 },
}

impl MarketType {
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

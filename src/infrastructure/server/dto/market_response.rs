use serde::Serialize;

use crate::domain::{Market, entities::Odd};

#[derive(Serialize)]
pub struct OddResponse {
    pub value: f64,
}
impl From<&Odd> for OddResponse {
    fn from(o: &Odd) -> Self {
        OddResponse { value: o.get() }
    }
}
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum MarketResponse {
    MatchResult {
        home: OddResponse,
        draw: OddResponse,
        away: OddResponse,
    },
    Moneyline {
        home: OddResponse,
        away: OddResponse,
    },
    DoubleChance {
        home_or_draw: OddResponse,
        home_or_away: OddResponse,
        draw_or_away: OddResponse,
    },
    Total {
        line: f32,
        over: OddResponse,
        under: OddResponse,
    },
    Handicap {
        line: f32,
        home: OddResponse,
        draw: OddResponse,
        away: OddResponse,
    },
    AsianHandicap {
        line: f32,
        home: OddResponse,
        away: OddResponse,
    },
}
impl From<&Market> for MarketResponse {
    fn from(m: &Market) -> Self {
        match m {
            Market::MatchResult(m) => MarketResponse::MatchResult {
                home: (&m.home).into(),
                draw: (&m.draw).into(),
                away: (&m.away).into(),
            },
            Market::Moneyline(m) => MarketResponse::Moneyline {
                home: (&m.home).into(),
                away: (&m.away).into(),
            },
            Market::DoubleChance(m) => MarketResponse::DoubleChance {
                home_or_draw: (&m.home_or_draw).into(),
                home_or_away: (&m.home_or_away).into(),
                draw_or_away: (&m.draw_or_away).into(),
            },
            Market::Total(m) => MarketResponse::Total {
                line: m.line.0,
                over: (&m.over).into(),
                under: (&m.under).into(),
            },
            Market::Handicap(m) => MarketResponse::Handicap {
                line: m.line.0,
                home: (&m.home).into(),
                draw: (&m.draw).into(),
                away: (&m.away).into(),
            },
            Market::AsianHandicap(m) => MarketResponse::AsianHandicap {
                line: m.line.0,
                home: (&m.home).into(),
                away: (&m.away).into(),
            },
        }
    }
}

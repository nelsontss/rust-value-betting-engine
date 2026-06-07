use serde::Serialize;

use crate::domain::{Game, Market, entities::Odd};
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
#[derive(Serialize)]
pub struct GameResponse {
    pub id: String,
    pub home_team: String,
    pub away_team: String,
    pub country: String,
    pub competition: String,
    pub platform: String,
    pub date: String,
    pub markets: Vec<MarketResponse>,
}
impl From<&Game> for GameResponse {
    fn from(g: &Game) -> Self {
        GameResponse {
            id: g.id.clone(),
            home_team: g.home_team().to_string(),
            away_team: g.away_team().to_string(),
            country: g.country().to_string(),
            competition: g.competition().to_string(),
            platform: format!("{:?}", g.platform()),
            date: g.date.to_string(),
            markets: g.markets().values().map(|m| m.into()).collect(),
        }
    }
}

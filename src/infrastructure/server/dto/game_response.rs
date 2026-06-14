use serde::Serialize;

use crate::{domain::Game, infrastructure::server::dto::market_response::MarketResponse};

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
impl From<Game> for GameResponse {
    fn from(g: Game) -> Self {
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

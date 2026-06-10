use std::sync::Arc;

use serde::Serialize;

use crate::{
    domain::entities::FixtureCluster, infrastructure::server::dto::game_response::GameResponse,
};

#[derive(Serialize)]
pub struct ClusterResponse {
    pub id: String,
    pub games: Vec<GameResponse>,
    pub representative_game: Option<GameResponse>,
    pub updated_at: String,
}
impl From<&Arc<FixtureCluster>> for ClusterResponse {
    fn from(c: &Arc<FixtureCluster>) -> Self {
        ClusterResponse {
            id: c.key(),
            representative_game: c
                .representative_game()
                .and_then(|game| Some(GameResponse::from(game))),
            games: c.games().into_iter().map(GameResponse::from).collect(),
            updated_at: c.updated_at().to_rfc3339(),
        }
    }
}

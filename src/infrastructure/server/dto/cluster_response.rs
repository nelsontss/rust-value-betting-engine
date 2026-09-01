use std::{collections::HashMap, sync::Arc};

use serde::Serialize;

use crate::{
    domain::entities::FixtureCluster, infrastructure::server::dto::game_response::GameResponse,
};

#[derive(Serialize)]
pub struct ClusterResponse {
    pub id: String,
    pub games: Vec<GameResponse>,
    pub representative_game: Option<GameResponse>,
    pub live_diffs: HashMap<String, HashMap<String, f64>>,
    pub updated_at: String,
}
impl From<&Arc<FixtureCluster>> for ClusterResponse {
    fn from(c: &Arc<FixtureCluster>) -> Self {
        let mut live_diffs: HashMap<String, HashMap<String, f64>> = HashMap::new();
        for (mt, inner) in c.live_statistics_diffs() {
            for (out, diff) in inner {
                live_diffs
                    .entry(mt.to_key_string())
                    .or_default()
                    .insert(format!("{:?}", out), diff);
            }
        }
        ClusterResponse {
            id: c.key(),
            representative_game: c
                .representative_game()
                .map(|game| GameResponse::from(game.clone())),
            games: c.games().map(|g| GameResponse::from(g.clone())).collect(),
            live_diffs,
            updated_at: c.updated_at().to_rfc3339(),
        }
    }
}

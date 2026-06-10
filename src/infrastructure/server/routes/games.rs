use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use axum_macros::debug_handler;
use tokio::sync::RwLock;

use crate::{
    domain::{ClusterService, Platform},
    infrastructure::server::dto::game_response::GameResponse,
};

#[debug_handler]
pub async fn get(
    State(cluster_service): State<Arc<RwLock<ClusterService>>>,
) -> Json<Vec<GameResponse>> {
    let response: Vec<GameResponse> = cluster_service
        .read()
        .await
        .get_games()
        .map(|g| GameResponse::from(g))
        .collect();
    Json(response)
}

#[debug_handler]
pub async fn get_by_platform(
    State(cluster_service): State<Arc<RwLock<ClusterService>>>,
    Path(platform): Path<Platform>,
) -> Json<Vec<GameResponse>> {
    let response: Vec<GameResponse> = cluster_service
        .read()
        .await
        .get_plaftorm_games(&platform)
        .map(|g| GameResponse::from(g))
        .collect();
    Json(response)
}

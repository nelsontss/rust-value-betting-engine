use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use axum_macros::debug_handler;

use crate::{
    domain::Platform,
    infrastructure::server::{dto::game_response::GameResponse, routes::routes::AppState},
};

#[debug_handler]
pub async fn get(State(app_state): State<Arc<AppState>>) -> Json<Vec<GameResponse>> {
    let response: Vec<GameResponse> = app_state
        .cluster_service
        .get_games()
        .into_iter()
        .map(|g| GameResponse::from(g))
        .collect();
    Json(response)
}

#[debug_handler]
pub async fn get_by_platform(
    State(app_state): State<Arc<AppState>>,
    Path(platform): Path<Platform>,
) -> Json<Vec<GameResponse>> {
    let response: Vec<GameResponse> = app_state
        .cluster_service
        .get_plaftorm_games(&platform)
        .into_iter()
        .map(|g| GameResponse::from(g))
        .collect();
    Json(response)
}

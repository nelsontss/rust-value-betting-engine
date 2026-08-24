use std::sync::Arc;

use axum::{Router, http::Method, routing::get};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    domain::{
        ClusterService,
        services::{market_service::MarketService, statistics_service::StatisticsService},
    },
    infrastructure::server::routes::{clusters, games, market_history, platforms, statistics},
};

pub struct AppState {
    pub cluster_service: Arc<ClusterService>,
    pub market_service: Arc<MarketService>,
    pub statistics_service: Arc<StatisticsService>,
}

pub fn build_router(app_state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_headers(Any);

    Router::new()
        .route("/clusters", get(clusters::get))
        .route("/clusters/{id}", get(clusters::get_by_id))
        .route("/sse/clusters", get(clusters::sse_get))
        .route("/games", get(games::get))
        .route("/games/{platform}", get(games::get_by_platform))
        .route("/platforms", get(platforms::get))
        .route("/games/{id}/markets/history", get(market_history::get))
        .route(
            "/see/games/{id}/markets/history",
            get(market_history::sse_get),
        )
        .route("/statistics", get(statistics::sse_get))
        .layer(cors)
        .with_state(app_state)
}

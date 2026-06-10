use std::sync::Arc;

use axum::{Router, http::Method, routing::get};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    domain::ClusterService,
    infrastructure::server::routes::{clusters, games},
};

pub fn build_router(cluster_service: Arc<RwLock<ClusterService>>) -> Router {
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
        .layer(cors)
        .with_state(cluster_service)
}

use std::sync::Arc;

use crate::infrastructure::server::routes::routes::{AppState, build_router};

pub async fn serve(app_state: Arc<AppState>, shutdown: impl std::future::Future<Output = ()> + Send + 'static) {
    let app = build_router(app_state);
    let port = std::env::var("PORT").unwrap_or_else(|_| "3005".into());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}",))
        .await
        .unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .unwrap();
}

pub mod dto;
pub mod routes;

use std::sync::Arc;

use tokio::{signal, sync::RwLock};

use crate::{domain::ClusterService, infrastructure::server::routes::routes::build_router};

pub async fn serve(cluster_service: Arc<RwLock<ClusterService>>) {
    let app = build_router(cluster_service);
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}",))
        .await
        .unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

pub mod dto;
pub mod routes;

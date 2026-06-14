use std::sync::Arc;

use rust_value_betting_engine::{
    application::services::bookmaker_scrapper_service::BookmakerScrapperService,
    domain::{ClusterService, services::market_history_service::MarketHistoryService},
    infrastructure::server::routes::routes::AppState,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let market_history_service = Arc::new(MarketHistoryService::default());
    let cluster_service = Arc::new(ClusterService::with_market_history_service(Arc::clone(
        &market_history_service,
    )));
    let cs_bookmaker_clone = cluster_service.clone();
    let engine = tokio::spawn(async move {
        let mut bookmaker_scrapper_service = BookmakerScrapperService::new(cs_bookmaker_clone);

        bookmaker_scrapper_service.run().await;
    });
    let app_state = Arc::new(AppState {
        cluster_service,
        market_history_service,
    });
    let server = tokio::spawn(rust_value_betting_engine::infrastructure::server::serve(
        app_state,
    ));

    tokio::select! {
        _ = engine => tracing::warn!("engine stopped"),
        _ = tokio::signal::ctrl_c() => tracing::info!("received ctrl+c, shutting down"),
    }

    server.abort();
}

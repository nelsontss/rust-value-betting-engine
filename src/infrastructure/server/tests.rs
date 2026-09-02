use std::sync::Arc;
use std::time::Duration;

use uuid::Uuid;

use crate::domain::{
    ClusterService,
    services::{alert_service::AlertService, market_service::MarketService, statistics_service::StatisticsService},
};
use crate::infrastructure::repositories::{
    connect_pool,
    fixture_cluster_repository::FixtureClusterRepository,
    game_repository::GameRepository,
};
use crate::infrastructure::server::routes::routes::AppState;

use super::serve;

#[tokio::test]
async fn serve_binds_shutdown_and_stops_cleanly() {
    let db_path = format!(
        "{}/server_serve_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    let pool = connect_pool(&db_path).await.unwrap();
    let game_repo = Arc::new(GameRepository::from_pool(pool.clone()));
    let cluster_repo = Arc::new(FixtureClusterRepository::from_pool(pool.clone(), game_repo));
    cluster_repo.run_migrations().await.unwrap();

    let market_service = Arc::new(MarketService::new(Arc::new(GameRepository::from_pool(
        pool.clone(),
    ))));
    let statistics_service = Arc::new(StatisticsService::new(cluster_repo));
    let cluster_service = Arc::new(ClusterService::new());
    let alert_service = Arc::new(AlertService::new(
        cluster_service.clone(),
        statistics_service.clone(),
    ));

    let app_state = Arc::new(AppState {
        cluster_service,
        market_service,
        statistics_service,
        alert_service,
    });

    // unique port so parallel test runs never collide; serve reads PORT once
    let port = 30000 + (Uuid::new_v4().as_u128() as u16 % 20000);
    // SAFETY: single-threaded test setup, no other threads read the env yet
    unsafe { std::env::set_var("PORT", port.to_string()) };

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(serve(app_state, async move {
        let _ = shutdown_rx.await;
    }));

    // give the listener a moment to come up
    tokio::time::sleep(Duration::from_millis(300)).await;

    // the endpoint answers before shutdown
    let response = reqwest::get(format!("http://127.0.0.1:{port}/platforms")).await;
    assert!(response.is_ok(), "expected the server to answer /platforms");

    // trigger graceful shutdown and assert the server task finishes
    let _ = shutdown_tx.send(());
    let result = tokio::time::timeout(Duration::from_secs(5), server).await;
    assert!(result.is_ok(), "server did not shut down in time");
}

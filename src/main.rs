use std::{env, sync::Arc};

use rust_value_betting_engine::{
    application::services::bookmaker_scrapper_service::BookmakerScrapperService,
    domain::{
        ClusterService,
        services::{market_service::MarketService, statistics_service::StatisticsService},
    },
    infrastructure::{
        repositories::{
            connect_pool, fixture_cluster_repository::FixtureClusterRepository,
            game_repository::GameRepository,
        },
        server::routes::routes::AppState,
    },
};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let pool = connect_pool(&env::var("DB_PATH").expect("DB_PATH not defined"))
        .await
        .expect("Could not initialize db pool");
    let game_repository = Arc::new(GameRepository::from_pool(pool.clone()));
    let fixture_cluster_repository = Arc::new(FixtureClusterRepository::from_pool(
        pool,
        Arc::clone(&game_repository),
    ));

    fixture_cluster_repository
        .run_migrations()
        .await
        .expect("Error running migrations");

    let statistics_service = Arc::new(StatisticsService::new(Arc::clone(
        &fixture_cluster_repository,
    )));
    statistics_service
        .load_historical_diffs()
        .await
        .expect("Failed to load historical diffs");

    let market_service = Arc::new(MarketService::new(Arc::clone(&game_repository)));
    let cluster_service = Arc::new(
        ClusterService::new()
            .with_market_service(Arc::clone(&market_service))
            .with_fixture_cluster_repository(Arc::clone(&fixture_cluster_repository))
            .with_statistics_service(Arc::clone(&statistics_service)),
    );
    cluster_service.start_end_of_game_sweeper();
    let cs_bookmaker_clone = cluster_service.clone();
    let engine = tokio::spawn(async move {
        let mut bookmaker_scrapper_service = BookmakerScrapperService::new(cs_bookmaker_clone);

        bookmaker_scrapper_service.run().await;
    });
    let mut engine = engine;
    let app_state = Arc::new(AppState {
        cluster_service,
        market_service,
        statistics_service,
    });
    let shutdown = async {
        tokio::signal::ctrl_c().await.expect("failed to listen for ctrl+c");
    };
    let server = tokio::spawn(rust_value_betting_engine::infrastructure::server::serve(
        app_state.clone(),
        shutdown,
    ));

    tokio::select! {
        _ = &mut engine => tracing::warn!("engine stopped"),
        _ = tokio::signal::ctrl_c() => tracing::info!("received ctrl+c, shutting down"),
    }

    engine.abort();
    server.abort();
    app_state.cluster_service.flush_pending_persists().await;
    tracing::info!("shutdown complete");
    std::process::exit(0);
}

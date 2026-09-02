use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

use crate::domain::{
    ClusterService,
    entities::{Game, Market, Odd, Platform, markets::moneyline::MoneylineMarket},
    services::{alert_service::AlertService, market_service::MarketService, statistics_service::StatisticsService},
};
use crate::infrastructure::repositories::{
    connect_pool,
    fixture_cluster_repository::FixtureClusterRepository,
    game_repository::GameRepository,
};
use crate::infrastructure::server::routes::routes::{AppState, build_router};

async fn app_state() -> (Arc<AppState>, Arc<GameRepository>) {
    let db_path = format!(
        "{}/routes_test_{}.db",
        std::env::temp_dir().display(),
        Uuid::new_v4()
    );
    let pool = connect_pool(&db_path).await.unwrap();
    let game_repo = Arc::new(GameRepository::from_pool(pool.clone()));
    let cluster_repo = Arc::new(FixtureClusterRepository::from_pool(pool, game_repo.clone()));
    cluster_repo.run_migrations().await.unwrap();

    let market_service = Arc::new(MarketService::new(game_repo.clone()));
    let statistics_service = Arc::new(StatisticsService::new(cluster_repo.clone()));
    let cluster_service = Arc::new(
        ClusterService::new()
            .with_market_service(market_service.clone())
            .with_fixture_cluster_repository(cluster_repo)
            .with_statistics_service(statistics_service.clone()),
    );
    let alert_service = Arc::new(AlertService::new(
        cluster_service.clone(),
        statistics_service.clone(),
    ));

    (
        Arc::new(AppState {
            cluster_service,
            market_service,
            statistics_service,
            alert_service,
        }),
        game_repo,
    )
}

fn moneyline_game(id: &str, platform: Platform) -> Game {
    Game::new_with_id(
        id,
        "Benfica",
        "Sporting",
        "Portugal",
        "Primeira Liga",
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            chrono::NaiveTime::from_hms_milli_opt(20, 0, 0, 0).unwrap(),
        ),
        platform,
        vec![Market::Moneyline(MoneylineMarket::new(
            format!("{}-ml", id),
            Odd::new(2.0).unwrap(),
            Odd::new(1.8).unwrap(),
        ))],
        None,
    )
}

async fn get_json(uri: &str) -> (StatusCode, Value) {
    let (state, _) = app_state().await;
    let app = build_router(state);
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body).unwrap_or(Value::Null)
    };
    (status, value)
}

#[tokio::test]
async fn platforms_lists_every_platform_in_lowercase() {
    let (status, body) = get_json("/platforms").await;

    assert_eq!(StatusCode::OK, status);
    assert_eq!(
        json!(["betano", "lebull", "bwin", "polymarket"]),
        body
    );
}

#[tokio::test]
async fn games_returns_every_clustered_game() {
    let (state, _) = app_state().await;
    state
        .cluster_service
        .insert_games(vec![moneyline_game("g1", Platform::Betano)]);
    state
        .cluster_service
        .insert_games(vec![moneyline_game("g2", Platform::Polymarket)]);

    let app = build_router(state);
    let response = app
        .oneshot(Request::builder().uri("/games").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let games: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(2, games.len());
    let mut platforms: Vec<&str> = games
        .iter()
        .map(|g| g["platform"].as_str().unwrap())
        .collect();
    platforms.sort_unstable();
    assert_eq!(vec!["Betano", "Polymarket"], platforms);
}

#[tokio::test]
async fn games_by_platform_filters_games() {
    let (state, _) = app_state().await;
    state
        .cluster_service
        .insert_games(vec![moneyline_game("g1", Platform::Betano)]);
    state
        .cluster_service
        .insert_games(vec![moneyline_game("g2", Platform::Polymarket)]);

    let app = build_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/games/betano")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let games: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(1, games.len());
    assert_eq!("g1", games[0]["id"]);
}

#[tokio::test]
async fn games_by_platform_rejects_unknown_platform() {
    let (status, _) = get_json("/games/unknown-platform").await;

    assert_eq!(StatusCode::BAD_REQUEST, status);
}

#[tokio::test]
async fn clusters_returns_empty_list_without_multi_game_clusters() {
    let (state, _) = app_state().await;
    state
        .cluster_service
        .insert_games(vec![moneyline_game("g1", Platform::Betano)]);

    let app = build_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/clusters")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let clusters: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert!(clusters.is_empty());
}

#[tokio::test]
async fn cluster_by_id_returns_404_for_unknown_cluster() {
    let (status, _) = get_json("/clusters/does-not-exist").await;

    assert_eq!(StatusCode::NOT_FOUND, status);
}

#[tokio::test]
async fn cluster_by_id_returns_cluster_payload() {
    let (state, _) = app_state().await;
    state
        .cluster_service
        .insert_games(vec![moneyline_game("g1", Platform::Betano)]);
    state
        .cluster_service
        .insert_games(vec![moneyline_game("g2", Platform::Polymarket)]);
    let cluster_id = state.cluster_service.get_clusters()[0].key();
    let encoded = cluster_id.replace(' ', "%20").replace('/', "%2F");

    let app = build_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/clusters/{encoded}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let cluster: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(2, cluster["games"].as_array().unwrap().len());
}

#[tokio::test]
async fn market_history_returns_404_for_unknown_game() {
    let (status, _) = get_json("/games/unknown-game/markets/history").await;

    assert_eq!(StatusCode::NOT_FOUND, status);
}

#[tokio::test]
async fn market_history_serves_stored_market_points() {
    let (state, game_repo) = app_state().await;
    let game = moneyline_game("g1", Platform::Betano);
    game_repo.insert_game(&game).await.unwrap();

    let app = build_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/games/g1/markets/history")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, response.status());
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let history: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!("g1", history["game_id"]);
    assert_eq!(
        1,
        history["markets_by_type"]["Moneyline"]
            .as_array()
            .unwrap()
            .len()
    );
}

#[tokio::test]
async fn debug_memory_reports_process_memory() {
    let (status, body) = get_json("/debug/memory").await;

    assert_eq!(StatusCode::OK, status);
    assert!(
        body["memory"]["physical_mem"].is_u64() || body["memory"].is_null(),
        "unexpected memory payload: {body}"
    );
}

#[tokio::test]
async fn sse_endpoints_respond_with_event_stream_content_type() {
    let (state, _) = app_state().await;
    for uri in ["/sse/alerts", "/alerts", "/statistics", "/sse/clusters"] {
        let app = build_router(state.clone());
        let response = app
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(StatusCode::OK, response.status(), "uri {uri}");
        assert_eq!(
            "text/event-stream",
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
}


#[tokio::test]
async fn market_history_sse_endpoint_responds_with_event_stream() {
    let (state, _) = app_state().await;

    let app = build_router(state);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/see/games/some-game/markets/history")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(
        "text/event-stream",
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
    );
}

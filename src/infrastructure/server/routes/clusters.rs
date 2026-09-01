use std::{convert::Infallible, sync::Arc};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use axum_macros::debug_handler;
use futures::Stream;
use tokio::sync::broadcast::error::RecvError;

use crate::infrastructure::server::{
    dto::cluster_response::ClusterResponse, routes::routes::AppState,
};

#[debug_handler]
pub async fn get(State(app_state): State<Arc<AppState>>) -> Json<Vec<ClusterResponse>> {
    let fixtures = app_state.cluster_service.get_clusters();
    let response: Vec<ClusterResponse> =
        fixtures.iter().map(|c| ClusterResponse::from(c)).collect();
    Json(response)
}

#[debug_handler]
pub async fn get_by_id(
    State(app_state): State<Arc<AppState>>,
    Path(cluster_id): Path<String>,
) -> Result<Json<ClusterResponse>, StatusCode> {
    let cluster = app_state.cluster_service.get_cluster(&cluster_id);
    match cluster {
        Ok(cluster) => Ok(Json(ClusterResponse::from(&cluster))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[debug_handler]
pub async fn sse_get(
    State(app_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx: tokio::sync::broadcast::Receiver<Arc<crate::domain::entities::FixtureCluster>> =
        app_state.cluster_service.subscribe_to_cluster_updates();
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(cluster) => {
                  let response = ClusterResponse::from(&cluster);
                  yield Ok(Event::default().data(serde_json::to_string(&response).unwrap()))
                },
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

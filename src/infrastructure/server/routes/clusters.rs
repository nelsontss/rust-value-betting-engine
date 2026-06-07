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
use tokio::sync::{RwLock, broadcast::error::RecvError};

use crate::{
    domain::ClusterService, infrastructure::server::dto::cluster_response::ClusterResponse,
};

#[debug_handler]
pub async fn get(
    State(cluster_service): State<Arc<RwLock<ClusterService>>>,
) -> Json<Vec<ClusterResponse>> {
    let fixtures = cluster_service.read().await.get_clusters();
    let response: Vec<ClusterResponse> =
        fixtures.iter().map(|c| ClusterResponse::from(c)).collect();
    Json(response)
}

#[debug_handler]
pub async fn get_by_id(
    State(cluster_service): State<Arc<RwLock<ClusterService>>>,
    Path(cluster_id): Path<String>,
) -> Result<Json<ClusterResponse>, StatusCode> {
    let cluster = cluster_service.read().await.get_cluster(&cluster_id);
    match cluster {
        Ok(cluster) => Ok(Json(ClusterResponse::from(&cluster))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[debug_handler]
pub async fn sse_get(
    State(cluster_service): State<Arc<RwLock<ClusterService>>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = cluster_service.read().await.subscribe_to_game_updates();
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(game) => {
                  let response = ClusterResponse::from(&game);
                  yield Ok(Event::default().data(serde_json::to_string(&response).unwrap()))
                },
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

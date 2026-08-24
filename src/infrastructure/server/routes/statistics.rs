use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::State,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use axum_macros::debug_handler;
use futures::Stream;
use tokio::sync::broadcast::error::RecvError;

use crate::{
    domain::services::cluster_statistics::StatisticsUpdated,
    infrastructure::server::{
        dto::statistics_response::StatisticsUpdatedResponse, routes::routes::AppState,
    },
};

#[debug_handler]
pub async fn sse_get(
    State(app_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // subscribe before snapshotting so no completion event is missed in between
    let mut rx = app_state.statistics_service.subscribe_to_statistics();
    let initial = StatisticsUpdated {
        statistics: app_state.statistics_service.get_historical_statistics(),
    };
    let initial_response = StatisticsUpdatedResponse::from(&initial);

    let stream = async_stream::stream! {
        yield Ok(Event::default()
            .event("StatisticsUpdated")
            .data(serde_json::to_string(&initial_response).unwrap()));

        loop {
            match rx.recv().await {
                Ok(update) => {
                    let response = StatisticsUpdatedResponse::from(update.as_ref());
                    yield Ok(Event::default()
                        .event("StatisticsUpdated")
                        .data(serde_json::to_string(&response).unwrap()))
                },
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

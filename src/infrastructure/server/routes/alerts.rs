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

use crate::infrastructure::server::{
    dto::alert_response::AlertResponse, routes::routes::AppState,
};

#[debug_handler]
pub async fn sse_get(
    State(app_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = app_state.alert_service.subscribe_to_new_alerts();
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let response = AlertResponse::from(event.as_ref());
                    yield Ok(Event::default()
                        .event("Alert")
                        .data(serde_json::to_string(&response).unwrap()))
                },
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

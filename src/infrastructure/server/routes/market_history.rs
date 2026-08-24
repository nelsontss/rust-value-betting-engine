use std::{convert::Infallible, sync::Arc};

use axum::{
    Json,
    extract::{Path, State},
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use axum_macros::debug_handler;
use futures::Stream;
use reqwest::StatusCode;
use tokio::sync::broadcast::error::RecvError;

use crate::infrastructure::server::{
    dto::market_history_response::{MarketDataPointResponse, MarketHistoryResponse},
    routes::routes::AppState,
};

#[debug_handler]
pub async fn get(
    State(app_state): State<Arc<AppState>>,
    Path(game_id): Path<String>,
) -> Result<Json<MarketHistoryResponse>, StatusCode> {
    match app_state
        .market_service
        .get_game_markets_history(&game_id)
        .await
    {
        Ok(Some(markets)) => Ok(Json(MarketHistoryResponse::from((
            game_id.as_str(),
            &markets,
        )))),
        Ok(None) => {
            println!("Game not found");

            Err(StatusCode::NOT_FOUND)
        }
        Err(err) => {
            tracing::error!(
                error = %err,
                game_id = %game_id,
                "failed to load market history"
            );

            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn sse_get(
    State(app_state): State<Arc<AppState>>,
    Path(game_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = app_state.market_service.subscribe_to_game_market_updates();
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok((id, market_history)) if id == game_id => {
                  let response = MarketDataPointResponse::from((id.as_str(), &market_history));
                  yield Ok(Event::default().data(serde_json::to_string(&response).unwrap()))
                },
                Ok(_) => continue,
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

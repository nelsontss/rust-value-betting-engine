use axum::Json;

use crate::domain::Platform;
use strum::IntoEnumIterator;

pub async fn get() -> Json<Vec<Platform>> {
    Json(Platform::iter().collect())
}

use axum::Json;
use serde_json::{Value, json};

pub async fn memory() -> Json<Value> {
    let stats = memory_stats::memory_stats().map(|s| {
        json!({
            "physical_mem": s.physical_mem,
            "virtual_mem": s.virtual_mem
        })
    });
    Json(json!({ "memory": stats }))
}

use axum::Json;
use serde_json::Value;

pub async fn handler() -> Json<Value> {
    Json(serde_json::json!({"status": "not implemented"}))
}

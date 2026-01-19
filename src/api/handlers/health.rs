use axum::{Json, http::StatusCode};

pub async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "ok",
        "service": "atom_node",
        "version": "0.1.0"
    })))
}

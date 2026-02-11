use crate::state::AppState;
use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
}

pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    // Check if DB is alive using simple query
    let db_status = if sqlx::query("SELECT 1").execute(&state.db).await.is_ok() {
        "Connected"
    } else {
        "Disconnected"
    };

    let response = HealthResponse {
        status: "Mosaic Registry is Healthy!".to_string(),
        database: db_status.to_string(),
    };

    (StatusCode::OK, Json(response))
}

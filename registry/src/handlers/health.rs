use crate::state::AppState;
use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
}

/// Simple health check endpoint.
///
/// Used by load balancers and monitoring to know if the registry is still alive.
/// We ping the database to make sure the connection pool is working—if DB is down,
/// the whole registry is useless anyway.
pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    // Hit the database with a dummy query. `SELECT 1` is the fastest way to check if it's responsive.
    // If this fails, the DB is either down or the connection pool is maxed out.
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

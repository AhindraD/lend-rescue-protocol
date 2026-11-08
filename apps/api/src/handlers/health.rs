use axum::extract::State;
use axum::Json;
use chrono::Utc;

use crate::dto::{HealthResponse, ReadyResponse};
use crate::state::AppState;

/// Application liveness probe.
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "System",
    responses(
        (status = 200, description = "Service is alive", body = HealthResponse)
    )
)]
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "lend-rescue-api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Application readiness probe indicating database connection status.
#[utoipa::path(
    get,
    path = "/api/v1/ready",
    tag = "System",
    responses(
        (status = 200, description = "Service dependency readiness status", body = ReadyResponse)
    )
)]
pub async fn ready_handler(State(state): State<AppState>) -> Json<ReadyResponse> {
    let db_status = if state.db_pool.is_connected() {
        "connected"
    } else if state.config.demo_mode {
        "in_memory_fallback"
    } else {
        "disconnected"
    };

    Json(ReadyResponse {
        ready: true,
        database: db_status.to_string(),
        demo_mode: state.config.demo_mode,
        timestamp: Utc::now().timestamp(),
    })
}

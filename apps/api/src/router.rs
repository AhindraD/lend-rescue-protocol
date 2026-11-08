use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::middleware as axum_middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::handlers::*;
use crate::middleware::request_id_middleware;
use crate::state::AppState;

/// Assembles the complete Axum application router with middleware and documentation.
pub fn create_router(state: AppState) -> Router {
    // 1. Versioned API routes (/api/v1)
    let api_v1 = Router::new()
        // Health & Readiness
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        // Markets
        .route("/markets", get(list_markets_handler))
        // Positions
        .route("/positions", get(list_positions_handler))
        .route("/positions/{position_id}", get(get_position_handler))
        .route(
            "/positions/{position_id}/risk",
            get(get_position_risk_handler),
        )
        // Oracles
        .route("/oracles/{asset_id}", get(get_oracle_price_handler))
        // Recovery
        .route("/recovery/quote", post(quote_recovery_handler))
        .route("/recovery/optimize", post(optimize_recovery_handler))
        .route(
            "/recovery/{recovery_id}",
            get(get_recovery_recommendation_handler),
        )
        // OpenAPI spec
        .route("/openapi.json", get(openapi_json_handler));

    // 2. CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any) // In development/demo, allow any origin; in production restricted by reverse proxy/config
        .allow_methods(Any)
        .allow_headers(Any);

    // 3. Assemble root router with middleware
    Router::new()
        .nest("/api/v1", api_v1)
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024)) // 2 MB body limit
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(state.config.request_timeout_secs),
        ))
        .layer(TraceLayer::new_for_http())
        .layer(axum_middleware::from_fn(request_id_middleware))
        .layer(cors)
        .with_state(state)
}

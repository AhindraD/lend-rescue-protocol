use axum::Json;
use utoipa::OpenApi;

use crate::dto::*;
use crate::error::{ErrorPayload, ErrorResponse};
use crate::handlers::{health, markets, oracles, positions, recovery};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_handler,
        health::ready_handler,
        markets::list_markets_handler,
        positions::list_positions_handler,
        positions::get_position_handler,
        positions::get_position_risk_handler,
        oracles::get_oracle_price_handler,
        recovery::quote_recovery_handler,
        recovery::optimize_recovery_handler,
        recovery::get_recovery_recommendation_handler,
    ),
    components(
        schemas(
            HealthResponse,
            ReadyResponse,
            MarketResponse,
            PositionResponse,
            TokenAmountDto,
            OraclePriceResponse,
            PriceDto,
            RiskAssessmentResponse,
            RecoveryQuoteRequest,
            RecoveryOptimizeRequest,
            RecoveryPlanDto,
            RecoveryRecommendationResponse,
            ScoreBreakdownDto,
            SwapQuoteDto,
            SwapRouteStepDto,
            ErrorResponse,
            ErrorPayload,
        )
    ),
    tags(
        (name = "System", description = "Health and readiness diagnostics"),
        (name = "Markets", description = "Lending market definitions"),
        (name = "Positions", description = "Position snapshots and risk evaluation"),
        (name = "Oracles", description = "Oracle price observations"),
        (name = "Recovery", description = "Recovery optimization and plan quotation")
    ),
    info(
        title = "LendRescue REST API",
        version = "0.1.0",
        description = "Deterministic lending risk engine and execution-aware recovery optimizer on Solana",
        license(name = "Apache-2.0")
    )
)]
pub struct ApiDoc;

/// Serves OpenAPI JSON schema.
pub async fn openapi_json_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

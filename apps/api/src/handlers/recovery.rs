use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use crate::dto::{
    RecoveryOptimizeRequest, RecoveryPlanDto, RecoveryQuoteRequest, RecoveryRecommendationResponse,
};
use crate::error::{ApiError, ErrorResponse};
use crate::state::AppState;

/// Quotes a single proposed recovery action for an at-risk position.
#[utoipa::path(
    post,
    path = "/api/v1/recovery/quote",
    tag = "Recovery",
    request_body = RecoveryQuoteRequest,
    responses(
        (status = 200, description = "Generated recovery plan quote", body = RecoveryPlanDto),
        (status = 400, description = "Invalid request parameters", body = ErrorResponse),
        (status = 404, description = "Position not found", body = ErrorResponse),
        (status = 422, description = "Infeasible recovery parameters", body = ErrorResponse)
    )
)]
pub async fn quote_recovery_handler(
    State(state): State<AppState>,
    Json(payload): Json<RecoveryQuoteRequest>,
) -> Result<Json<RecoveryPlanDto>, ApiError> {
    let plan = state.recovery_service.quote_recovery(payload).await?;
    Ok(Json(plan))
}

/// Evaluates and deterministically ranks all recovery options for an at-risk position.
#[utoipa::path(
    post,
    path = "/api/v1/recovery/optimize",
    tag = "Recovery",
    request_body = RecoveryOptimizeRequest,
    responses(
        (status = 200, description = "Ranked recovery recommendation response", body = RecoveryRecommendationResponse),
        (status = 400, description = "Invalid optimization parameters", body = ErrorResponse),
        (status = 404, description = "Position not found", body = ErrorResponse)
    )
)]
pub async fn optimize_recovery_handler(
    State(state): State<AppState>,
    Json(payload): Json<RecoveryOptimizeRequest>,
) -> Result<Json<RecoveryRecommendationResponse>, ApiError> {
    let recommendation = state.recovery_service.optimize_recovery(payload).await?;
    Ok(Json(recommendation))
}

/// Retrieves a stored recovery recommendation by UUID.
#[utoipa::path(
    get,
    path = "/api/v1/recovery/{recovery_id}",
    tag = "Recovery",
    params(
        ("recovery_id" = String, Path, description = "Recovery recommendation UUID")
    ),
    responses(
        (status = 200, description = "Stored recovery recommendation", body = RecoveryRecommendationResponse),
        (status = 400, description = "Invalid UUID format", body = ErrorResponse),
        (status = 404, description = "Recovery recommendation not found", body = ErrorResponse)
    )
)]
pub async fn get_recovery_recommendation_handler(
    State(state): State<AppState>,
    Path(recovery_id_str): Path<String>,
) -> Result<Json<RecoveryRecommendationResponse>, ApiError> {
    let recovery_id = Uuid::parse_str(&recovery_id_str)
        .map_err(|_| ApiError::BadRequest(format!("Invalid UUID string '{}'", recovery_id_str)))?;

    let recommendation = state
        .recovery_service
        .get_recommendation(recovery_id)
        .await?;

    Ok(Json(recommendation))
}

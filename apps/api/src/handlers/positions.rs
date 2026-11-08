use axum::extract::{Path, Query, State};
use axum::Json;

use crate::dto::{PositionFilterQuery, PositionResponse, RiskAssessmentResponse};
use crate::error::{ApiError, ErrorResponse};
use crate::state::AppState;

/// Lists lending positions with optional filters.
#[utoipa::path(
    get,
    path = "/api/v1/positions",
    tag = "Positions",
    params(PositionFilterQuery),
    responses(
        (status = 200, description = "List of normalized positions", body = Vec<PositionResponse>),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
pub async fn list_positions_handler(
    State(state): State<AppState>,
    Query(query): Query<PositionFilterQuery>,
) -> Result<Json<Vec<PositionResponse>>, ApiError> {
    let positions = state.position_service.list_positions(&query).await?;
    let dtos = positions
        .iter()
        .map(PositionResponse::from_domain)
        .collect();
    Ok(Json(dtos))
}

/// Retrieves a single position snapshot by ID.
#[utoipa::path(
    get,
    path = "/api/v1/positions/{position_id}",
    tag = "Positions",
    params(
        ("position_id" = String, Path, description = "Position unique identifier")
    ),
    responses(
        (status = 200, description = "Normalized position snapshot", body = PositionResponse),
        (status = 404, description = "Position not found", body = ErrorResponse)
    )
)]
pub async fn get_position_handler(
    State(state): State<AppState>,
    Path(position_id): Path<String>,
) -> Result<Json<PositionResponse>, ApiError> {
    let position = state.position_service.get_position(&position_id).await?;
    Ok(Json(PositionResponse::from_domain(&position)))
}

/// Evaluates real-time financial risk for a position using the pure Rust risk-engine.
#[utoipa::path(
    get,
    path = "/api/v1/positions/{position_id}/risk",
    tag = "Positions",
    params(
        ("position_id" = String, Path, description = "Position unique identifier")
    ),
    responses(
        (status = 200, description = "Calculated position risk assessment", body = RiskAssessmentResponse),
        (status = 404, description = "Position or oracle feed not found", body = ErrorResponse)
    )
)]
pub async fn get_position_risk_handler(
    State(state): State<AppState>,
    Path(position_id): Path<String>,
) -> Result<Json<RiskAssessmentResponse>, ApiError> {
    let risk = state
        .risk_service
        .assess_position_risk(&position_id)
        .await?;
    Ok(Json(risk))
}

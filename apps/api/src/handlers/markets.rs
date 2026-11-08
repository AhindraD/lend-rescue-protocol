use axum::extract::State;
use axum::Json;
use chrono::Utc;

use crate::dto::MarketResponse;
use crate::error::{ApiError, ErrorResponse};
use crate::state::AppState;

/// Lists all supported lending markets and parameters.
#[utoipa::path(
    get,
    path = "/api/v1/markets",
    tag = "Markets",
    responses(
        (status = 200, description = "List of lending markets", body = Vec<MarketResponse>),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
pub async fn list_markets_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<MarketResponse>>, ApiError> {
    let now = Utc::now().timestamp();
    let markets = state.market_service.list_markets().await?;

    let dtos: Vec<MarketResponse> = markets
        .into_iter()
        .map(|m| MarketResponse::from_domain(&m, now))
        .collect();

    Ok(Json(dtos))
}

use axum::extract::{Path, State};
use axum::Json;
use chrono::Utc;
use domain_types::AssetId;
use risk_engine::validate_oracle_price;

use crate::dto::OraclePriceResponse;
use crate::error::{ApiError, ErrorResponse};
use crate::state::AppState;

/// Retrieves current oracle price observation and freshness metadata for an asset.
#[utoipa::path(
    get,
    path = "/api/v1/oracles/{asset_id}",
    tag = "Oracles",
    params(
        ("asset_id" = String, Path, description = "Asset symbol or identifier (e.g. sSOL, sUSDC)")
    ),
    responses(
        (status = 200, description = "Oracle price observation", body = OraclePriceResponse),
        (status = 404, description = "Asset oracle not found", body = ErrorResponse)
    )
)]
pub async fn get_oracle_price_handler(
    State(state): State<AppState>,
    Path(asset_id_str): Path<String>,
) -> Result<Json<OraclePriceResponse>, ApiError> {
    let asset_id = match asset_id_str.to_ascii_uppercase().as_str() {
        "SOL" => AssetId::Sol,
        "SSOL" => AssetId::SSol,
        "USDC" => AssetId::Usdc,
        "SUSDC" => AssetId::SUsdc,
        other => AssetId::Custom(other.to_string()),
    };

    let observation = state
        .oracle_provider
        .get_observation(&asset_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "Oracle price not found for asset '{}'",
                asset_id_str
            ))
        })?;

    let now = Utc::now().timestamp();
    let is_valid = validate_oracle_price(&observation.price, now, None, None).is_ok();

    Ok(Json(OraclePriceResponse::new(
        asset_id_str,
        &observation.price,
        now,
        is_valid && observation.is_valid,
        observation.source.as_str(),
    )))
}

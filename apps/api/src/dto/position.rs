use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::dto::common::TokenAmountDto;

/// Query parameters for filtering positions.
#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct PositionFilterQuery {
    pub owner: Option<String>,
    pub protocol: Option<String>,
    pub network: Option<String>,
    pub risk_band: Option<String>,
}

/// Normalized position snapshot DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PositionResponse {
    pub id: String,
    pub owner: String,
    pub collateral_asset: String,
    pub collateral_amount: TokenAmountDto,
    pub debt_asset: String,
    pub debt_amount: TokenAmountDto,
    pub liquidation_threshold_bps: u16,
    pub protocol_source: String,
    pub provenance_label: String,
    pub market_id: Option<u64>,
}

impl PositionResponse {
    pub fn from_domain(pos: &domain_types::PositionSnapshot) -> Self {
        Self {
            id: pos.id.clone(),
            owner: pos.owner.clone(),
            collateral_asset: pos.collateral_asset.symbol().to_string(),
            collateral_amount: TokenAmountDto::from_domain(&pos.collateral_amount),
            debt_asset: pos.debt_asset.symbol().to_string(),
            debt_amount: TokenAmountDto::from_domain(&pos.debt_amount),
            liquidation_threshold_bps: pos.liquidation_threshold_bps,
            protocol_source: format!("{:?}", pos.protocol_source),
            provenance_label: pos.protocol_source.label().to_string(),
            market_id: pos.market_id,
        }
    }
}

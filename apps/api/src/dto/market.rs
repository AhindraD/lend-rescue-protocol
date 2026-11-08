use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Details describing a lending market.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MarketResponse {
    pub market_id: u64,
    pub authority: String,
    pub collateral_asset: String,
    pub collateral_decimals: u8,
    pub debt_asset: String,
    pub debt_decimals: u8,
    pub liquidation_threshold_bps: u16,
    pub demo_swap_fee_bps: u16,
    pub is_paused: bool,
    pub data_source: String,
    pub observed_at: i64,
}

impl MarketResponse {
    pub fn from_domain(market: &domain_types::MarketConfig, observed_at: i64) -> Self {
        Self {
            market_id: market.market_id,
            authority: market.authority.clone(),
            collateral_asset: market.collateral_asset.symbol().to_string(),
            collateral_decimals: market.collateral_asset.default_decimals(),
            debt_asset: market.debt_asset.symbol().to_string(),
            debt_decimals: market.debt_asset.default_decimals(),
            liquidation_threshold_bps: market.liquidation_threshold_bps,
            demo_swap_fee_bps: market.demo_swap_fee_bps,
            is_paused: market.is_paused,
            data_source: "Demo / Synthetic".to_string(),
            observed_at,
        }
    }
}

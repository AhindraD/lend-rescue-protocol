use serde::{Deserialize, Serialize};

use crate::asset::AssetId;

/// Configuration parameters for a lending market.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketConfig {
    pub market_id: u64,
    pub authority: String,
    pub collateral_asset: AssetId,
    pub debt_asset: AssetId,
    pub liquidation_threshold_bps: u16,
    pub demo_swap_fee_bps: u16,
    pub is_paused: bool,
}

impl MarketConfig {
    pub fn new(
        market_id: u64,
        authority: impl Into<String>,
        collateral_asset: AssetId,
        debt_asset: AssetId,
        liquidation_threshold_bps: u16,
        demo_swap_fee_bps: u16,
    ) -> Self {
        Self {
            market_id,
            authority: authority.into(),
            collateral_asset,
            debt_asset,
            liquidation_threshold_bps,
            demo_swap_fee_bps,
            is_paused: false,
        }
    }
}


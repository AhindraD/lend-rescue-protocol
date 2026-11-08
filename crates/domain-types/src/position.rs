use serde::{Deserialize, Serialize};

use crate::amount::TokenAmount;
use crate::asset::AssetId;

/// Source protocol or environment provenance for a position snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolSource {
    /// Devnet synthetic demo market
    SyntheticDemo,
    /// Kamino lending obligation observed on Solana Mainnet
    KaminoMainnet,
    /// Surfpool simulated local fork
    SurfpoolSimulated,
    /// Other protocol
    Custom(String),
}

impl ProtocolSource {
    pub fn label(&self) -> &str {
        match self {
            ProtocolSource::SyntheticDemo => "Devnet Executed",
            ProtocolSource::KaminoMainnet => "Mainnet Observed",
            ProtocolSource::SurfpoolSimulated => "Surfpool Simulated",
            ProtocolSource::Custom(s) => s.as_str(),
        }
    }
}

/// Normalized snapshot of a collateralized debt position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionSnapshot {
    /// Unique position or obligation identifier
    pub id: String,
    /// Owner wallet address
    pub owner: String,
    /// Collateral asset identifier
    pub collateral_asset: AssetId,
    /// Collateral amount deposited in base units
    pub collateral_amount: TokenAmount,
    /// Debt asset identifier
    pub debt_asset: AssetId,
    /// Debt amount borrowed in base units
    pub debt_amount: TokenAmount,
    /// Liquidation threshold in basis points (e.g. 8500 = 85%)
    pub liquidation_threshold_bps: u16,
    /// Originating protocol source
    pub protocol_source: ProtocolSource,
    /// Market ID if applicable
    pub market_id: Option<u64>,
}

impl PositionSnapshot {
    /// Creates a synthetic demo position snapshot.
    pub fn synthetic_demo(
        id: impl Into<String>,
        owner: impl Into<String>,
        collateral_raw: u64,
        debt_raw: u64,
        liquidation_threshold_bps: u16,
    ) -> Self {
        Self {
            id: id.into(),
            owner: owner.into(),
            collateral_asset: AssetId::SSol,
            collateral_amount: TokenAmount::new(collateral_raw, 9, Some(AssetId::SSol)),
            debt_asset: AssetId::SUsdc,
            debt_amount: TokenAmount::new(debt_raw, 6, Some(AssetId::SUsdc)),
            liquidation_threshold_bps,
            protocol_source: ProtocolSource::SyntheticDemo,
            market_id: None,
        }
    }
}

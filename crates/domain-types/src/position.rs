use serde::{Deserialize, Serialize};

use crate::amount::TokenAmount;
use crate::asset::AssetId;
use crate::kamino::KaminoObligationSnapshot;

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

/// Individual collateral deposit in a multi-asset position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionDeposit {
    pub reserve_address: String,
    pub mint_address: String,
    pub asset_symbol: String,
    pub decimals: u8,
    pub amount: TokenAmount,
    pub market_value_usd_scaled: u128,
}

/// Individual debt borrow in a multi-asset position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionBorrow {
    pub reserve_address: String,
    pub mint_address: String,
    pub asset_symbol: String,
    pub decimals: u8,
    pub amount: TokenAmount,
    pub market_value_usd_scaled: u128,
}

/// Protocol-calculated official risk metrics (e.g. from Kamino).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolRiskMetrics {
    pub borrow_limit_usd_scaled: u128,
    pub liquidation_threshold_usd_scaled: u128,
    pub total_deposit_usd_scaled: u128,
    pub total_borrow_usd_scaled: u128,
    pub loan_to_value_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub borrow_utilization_bps: u16,
    pub net_account_value_usd_scaled: i128,
    pub is_protocol_reported: bool,
}

/// Normalized snapshot of a collateralized debt position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionSnapshot {
    /// Unique position or obligation identifier
    pub id: String,
    /// Owner wallet address
    pub owner: String,
    /// Collateral asset identifier (primary or dominant collateral)
    pub collateral_asset: AssetId,
    /// Collateral amount deposited in base units
    pub collateral_amount: TokenAmount,
    /// Debt asset identifier (primary or dominant debt)
    pub debt_asset: AssetId,
    /// Debt amount borrowed in base units
    pub debt_amount: TokenAmount,
    /// Liquidation threshold in basis points (e.g. 8500 = 85%)
    pub liquidation_threshold_bps: u16,
    /// Originating protocol source
    pub protocol_source: ProtocolSource,
    /// Market ID if applicable
    pub market_id: Option<u64>,

    /// Multi-asset collateral positions (backward compatible)
    #[serde(default)]
    pub collateral_positions: Vec<PositionDeposit>,
    /// Multi-asset debt positions (backward compatible)
    #[serde(default)]
    pub debt_positions: Vec<PositionBorrow>,
    /// Official protocol-calculated risk metrics where available
    #[serde(default)]
    pub protocol_metrics: Option<ProtocolRiskMetrics>,
    /// On-chain market public key / address if applicable
    #[serde(default)]
    pub market_address: Option<String>,
    /// Observation slot if available
    #[serde(default)]
    pub observation_slot: Option<u64>,
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
        let collateral_amount = TokenAmount::new(collateral_raw, 9, Some(AssetId::SSol));
        let debt_amount = TokenAmount::new(debt_raw, 6, Some(AssetId::SUsdc));

        Self {
            id: id.into(),
            owner: owner.into(),
            collateral_asset: AssetId::SSol,
            collateral_amount: collateral_amount.clone(),
            debt_asset: AssetId::SUsdc,
            debt_amount: debt_amount.clone(),
            liquidation_threshold_bps,
            protocol_source: ProtocolSource::SyntheticDemo,
            market_id: None,
            collateral_positions: vec![PositionDeposit {
                reserve_address: "synthetic-ssol-reserve".to_string(),
                mint_address: "synthetic-ssol-mint".to_string(),
                asset_symbol: "sSOL".to_string(),
                decimals: 9,
                amount: collateral_amount,
                market_value_usd_scaled: 0,
            }],
            debt_positions: vec![PositionBorrow {
                reserve_address: "synthetic-susdc-reserve".to_string(),
                mint_address: "synthetic-susdc-mint".to_string(),
                asset_symbol: "sUSDC".to_string(),
                decimals: 6,
                amount: debt_amount,
                market_value_usd_scaled: 0,
            }],
            protocol_metrics: None,
            market_address: None,
            observation_slot: None,
        }
    }

    /// Converts a rich Kamino obligation snapshot into a normalized PositionSnapshot.
    pub fn from_kamino_obligation(obl: &KaminoObligationSnapshot) -> Self {
        let (collateral_asset, collateral_amount) = match obl.deposits.first() {
            Some(d) => (
                match d.token_symbol.to_ascii_uppercase().as_str() {
                    "USDC" => AssetId::SUsdc,
                    _ => AssetId::SSol,
                },
                d.raw_amount.clone(),
            ),
            None => (AssetId::SSol, TokenAmount::new(0, 9, Some(AssetId::SSol))),
        };

        let (debt_asset, debt_amount) = match obl.borrows.first() {
            Some(b) => (
                match b.token_symbol.to_ascii_uppercase().as_str() {
                    "SOL" => AssetId::SSol,
                    _ => AssetId::SUsdc,
                },
                b.raw_amount.clone(),
            ),
            None => (AssetId::SUsdc, TokenAmount::new(0, 6, Some(AssetId::SUsdc))),
        };

        let collateral_positions: Vec<PositionDeposit> = obl
            .deposits
            .iter()
            .map(|d| PositionDeposit {
                reserve_address: d.reserve_address.clone(),
                mint_address: d.mint_address.clone(),
                asset_symbol: d.token_symbol.clone(),
                decimals: d.decimals,
                amount: d.raw_amount.clone(),
                market_value_usd_scaled: d.market_value_usd_scaled,
            })
            .collect();

        let debt_positions: Vec<PositionBorrow> = obl
            .borrows
            .iter()
            .map(|b| PositionBorrow {
                reserve_address: b.reserve_address.clone(),
                mint_address: b.mint_address.clone(),
                asset_symbol: b.token_symbol.clone(),
                decimals: b.decimals,
                amount: b.raw_amount.clone(),
                market_value_usd_scaled: b.market_value_usd_scaled,
            })
            .collect();

        let protocol_metrics = ProtocolRiskMetrics {
            borrow_limit_usd_scaled: obl.protocol_metrics.borrow_limit_usd_scaled,
            liquidation_threshold_usd_scaled: obl.protocol_metrics.liquidation_threshold_usd_scaled,
            total_deposit_usd_scaled: obl.protocol_metrics.total_deposit_usd_scaled,
            total_borrow_usd_scaled: obl.protocol_metrics.total_borrow_usd_scaled,
            loan_to_value_bps: obl.protocol_metrics.loan_to_value_bps,
            liquidation_threshold_bps: obl.protocol_metrics.liquidation_threshold_bps,
            borrow_utilization_bps: obl.protocol_metrics.borrow_utilization_bps,
            net_account_value_usd_scaled: obl.protocol_metrics.net_account_value_usd_scaled,
            is_protocol_reported: obl.protocol_metrics.is_protocol_reported,
        };

        Self {
            id: obl.obligation_id.clone(),
            owner: obl.owner.clone(),
            collateral_asset,
            collateral_amount,
            debt_asset,
            debt_amount,
            liquidation_threshold_bps: obl.protocol_metrics.liquidation_threshold_bps,
            protocol_source: obl.protocol_source.clone(),
            market_id: None,
            collateral_positions,
            debt_positions,
            protocol_metrics: Some(protocol_metrics),
            market_address: Some(obl.market_id.clone()),
            observation_slot: Some(obl.observation_slot),
        }
    }
}

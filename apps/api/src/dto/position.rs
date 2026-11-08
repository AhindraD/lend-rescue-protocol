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

/// Normalized multi-asset collateral deposit entry DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PositionDepositDto {
    pub reserve_address: String,
    pub mint_address: String,
    pub asset_symbol: String,
    pub decimals: u8,
    pub amount: TokenAmountDto,
    pub market_value_usd: String,
}

/// Normalized multi-asset debt borrow entry DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PositionBorrowDto {
    pub reserve_address: String,
    pub mint_address: String,
    pub asset_symbol: String,
    pub decimals: u8,
    pub amount: TokenAmountDto,
    pub market_value_usd: String,
}

/// Protocol-reported official risk metrics DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolRiskMetricsDto {
    pub borrow_limit_usd: String,
    pub liquidation_threshold_usd: String,
    pub total_deposit_usd: String,
    pub total_borrow_usd: String,
    pub loan_to_value_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub borrow_utilization_bps: u16,
    pub net_account_value_usd: String,
    pub is_protocol_reported: bool,
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

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub collateral_positions: Vec<PositionDepositDto>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub debt_positions: Vec<PositionBorrowDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub protocol_metrics: Option<ProtocolRiskMetricsDto>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub market_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub observation_slot: Option<u64>,
}

fn format_scaled_6(raw: u128) -> String {
    let whole = raw / 1_000_000;
    let frac = raw % 1_000_000;
    format!("{}.{:06}", whole, frac)
}

fn format_signed_scaled_6(raw: i128) -> String {
    let sign = if raw < 0 { "-" } else { "" };
    let abs = raw.unsigned_abs();
    let whole = abs / 1_000_000;
    let frac = abs % 1_000_000;
    format!("{}{}.{:06}", sign, whole, frac)
}

impl PositionResponse {
    pub fn from_domain(pos: &domain_types::PositionSnapshot) -> Self {
        let collateral_positions: Vec<PositionDepositDto> = pos
            .collateral_positions
            .iter()
            .map(|d| PositionDepositDto {
                reserve_address: d.reserve_address.clone(),
                mint_address: d.mint_address.clone(),
                asset_symbol: d.asset_symbol.clone(),
                decimals: d.decimals,
                amount: TokenAmountDto::from_domain(&d.amount),
                market_value_usd: format_scaled_6(d.market_value_usd_scaled),
            })
            .collect();

        let debt_positions: Vec<PositionBorrowDto> = pos
            .debt_positions
            .iter()
            .map(|b| PositionBorrowDto {
                reserve_address: b.reserve_address.clone(),
                mint_address: b.mint_address.clone(),
                asset_symbol: b.asset_symbol.clone(),
                decimals: b.decimals,
                amount: TokenAmountDto::from_domain(&b.amount),
                market_value_usd: format_scaled_6(b.market_value_usd_scaled),
            })
            .collect();

        let protocol_metrics = pos
            .protocol_metrics
            .as_ref()
            .map(|m| ProtocolRiskMetricsDto {
                borrow_limit_usd: format_scaled_6(m.borrow_limit_usd_scaled),
                liquidation_threshold_usd: format_scaled_6(m.liquidation_threshold_usd_scaled),
                total_deposit_usd: format_scaled_6(m.total_deposit_usd_scaled),
                total_borrow_usd: format_scaled_6(m.total_borrow_usd_scaled),
                loan_to_value_bps: m.loan_to_value_bps,
                liquidation_threshold_bps: m.liquidation_threshold_bps,
                borrow_utilization_bps: m.borrow_utilization_bps,
                net_account_value_usd: format_signed_scaled_6(m.net_account_value_usd_scaled),
                is_protocol_reported: m.is_protocol_reported,
            });

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
            collateral_positions,
            debt_positions,
            protocol_metrics,
            market_address: pos.market_address.clone(),
            observation_slot: pos.observation_slot,
        }
    }
}

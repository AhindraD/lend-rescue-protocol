use domain_types::{
    AssetId, KaminoBorrowSnapshot, KaminoDepositSnapshot, KaminoMarketSnapshot,
    KaminoObligationSnapshot, KaminoProtocolMetrics, KaminoReserveSnapshot, PositionSnapshot,
    ProtocolSource, TokenAmount,
};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

fn parse_usd_scaled_6(val_str: &str) -> u128 {
    // Parse decimal string like "1500.25" or "1500" into micro-USD (6 decimals)
    let parts: Vec<&str> = val_str.trim().split('.').collect();
    let whole: u128 = parts[0].parse().unwrap_or(0);
    let frac: u128 = if parts.len() > 1 {
        let frac_str = parts[1];
        if frac_str.len() >= 6 {
            frac_str[..6].parse().unwrap_or(0)
        } else {
            let padded = format!("{:0<6}", frac_str);
            padded.parse().unwrap_or(0)
        }
    } else {
        0
    };
    whole.saturating_mul(1_000_000).saturating_add(frac)
}

fn parse_usd_signed_scaled_6(val_str: &str) -> i128 {
    let s = val_str.trim();
    let is_neg = s.starts_with('-');
    let abs_str = s.trim_start_matches('-');
    let abs_val = parse_usd_scaled_6(abs_str) as i128;
    if is_neg {
        -abs_val
    } else {
        abs_val
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaminoReserveResponseJson {
    pub reserve_id: String,
    pub liquidity_mint: String,
    pub collateral_mint: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub loan_to_value_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub liquidation_bonus_bps: u16,
    pub available_liquidity: String,
    pub total_deposits: String,
    pub total_borrows: String,
    pub borrow_interest_rate_bps: u16,
    pub supply_interest_rate_bps: u16,
    pub oracle_address: String,
    pub last_update_slot: u64,
    pub status: String,
}

impl KaminoReserveResponseJson {
    pub fn to_domain(&self) -> KaminoReserveSnapshot {
        let avail_raw: u64 = self.available_liquidity.parse().unwrap_or(0);
        let deps_raw: u64 = self.total_deposits.parse().unwrap_or(0);
        let brws_raw: u64 = self.total_borrows.parse().unwrap_or(0);

        KaminoReserveSnapshot {
            reserve_id: self.reserve_id.clone(),
            liquidity_mint: self.liquidity_mint.clone(),
            collateral_mint: self.collateral_mint.clone(),
            token_symbol: self.token_symbol.clone(),
            decimals: self.decimals,
            loan_to_value_bps: self.loan_to_value_bps,
            liquidation_threshold_bps: self.liquidation_threshold_bps,
            liquidation_bonus_bps: self.liquidation_bonus_bps,
            available_liquidity: TokenAmount::new(avail_raw, self.decimals, None),
            total_deposits: TokenAmount::new(deps_raw, self.decimals, None),
            total_borrows: TokenAmount::new(brws_raw, self.decimals, None),
            borrow_interest_rate_bps: self.borrow_interest_rate_bps,
            supply_interest_rate_bps: self.supply_interest_rate_bps,
            oracle_address: self.oracle_address.clone(),
            last_update_slot: self.last_update_slot,
            status: self.status.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaminoMarketResponseJson {
    pub market_id: String,
    pub owner: String,
    pub authority: String,
    pub program_id: String,
    pub network: String,
    pub observation_slot: u64,
    pub observed_at: String,
    pub reserves: Vec<KaminoReserveResponseJson>,
}

impl KaminoMarketResponseJson {
    pub fn to_domain(&self) -> KaminoMarketSnapshot {
        KaminoMarketSnapshot {
            market_id: self.market_id.clone(),
            owner: self.owner.clone(),
            authority: self.authority.clone(),
            program_id: self.program_id.clone(),
            network: self.network.clone(),
            observation_slot: self.observation_slot,
            observed_at: self.observed_at.clone(),
            reserves: self.reserves.iter().map(|r| r.to_domain()).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaminoDepositEntryJson {
    pub reserve_address: String,
    pub mint_address: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub raw_amount: String,
    pub market_value_usd: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaminoBorrowEntryJson {
    pub reserve_address: String,
    pub mint_address: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub raw_amount: String,
    pub market_value_usd: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaminoProtocolMetricsJson {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaminoObligationResponseJson {
    pub obligation_id: String,
    pub owner: String,
    pub market_id: String,
    pub observation_slot: u64,
    pub observed_at: String,
    pub protocol_source: String,
    pub deposits: Vec<KaminoDepositEntryJson>,
    pub borrows: Vec<KaminoBorrowEntryJson>,
    pub protocol_metrics: KaminoProtocolMetricsJson,
}

impl KaminoObligationResponseJson {
    pub fn to_domain(&self) -> Result<KaminoObligationSnapshot, ApiError> {
        let protocol_source = match self.protocol_source.to_ascii_lowercase().as_str() {
            "kaminomainnet" => ProtocolSource::KaminoMainnet,
            "surfpoolsimulated" => ProtocolSource::SurfpoolSimulated,
            _ => ProtocolSource::Custom(self.protocol_source.clone()),
        };

        let deposits: Vec<KaminoDepositSnapshot> = self
            .deposits
            .iter()
            .map(|d| {
                let raw_amt: u64 = d.raw_amount.parse().unwrap_or(0);
                let mv_scaled = parse_usd_scaled_6(&d.market_value_usd);
                let asset = match d.token_symbol.to_ascii_uppercase().as_str() {
                    "USDC" => Some(AssetId::Usdc),
                    "SOL" => Some(AssetId::Sol),
                    _ => None,
                };
                KaminoDepositSnapshot {
                    reserve_address: d.reserve_address.clone(),
                    mint_address: d.mint_address.clone(),
                    token_symbol: d.token_symbol.clone(),
                    decimals: d.decimals,
                    raw_amount: TokenAmount::new(raw_amt, d.decimals, asset),
                    market_value_usd_scaled: mv_scaled,
                }
            })
            .collect();

        let borrows: Vec<KaminoBorrowSnapshot> = self
            .borrows
            .iter()
            .map(|b| {
                let raw_amt: u64 = b.raw_amount.parse().unwrap_or(0);
                let mv_scaled = parse_usd_scaled_6(&b.market_value_usd);
                let asset = match b.token_symbol.to_ascii_uppercase().as_str() {
                    "USDC" => Some(AssetId::Usdc),
                    "SOL" => Some(AssetId::Sol),
                    _ => None,
                };
                KaminoBorrowSnapshot {
                    reserve_address: b.reserve_address.clone(),
                    mint_address: b.mint_address.clone(),
                    token_symbol: b.token_symbol.clone(),
                    decimals: b.decimals,
                    raw_amount: TokenAmount::new(raw_amt, b.decimals, asset),
                    market_value_usd_scaled: mv_scaled,
                }
            })
            .collect();

        let protocol_metrics = KaminoProtocolMetrics {
            borrow_limit_usd_scaled: parse_usd_scaled_6(&self.protocol_metrics.borrow_limit_usd),
            liquidation_threshold_usd_scaled: parse_usd_scaled_6(
                &self.protocol_metrics.liquidation_threshold_usd,
            ),
            total_deposit_usd_scaled: parse_usd_scaled_6(&self.protocol_metrics.total_deposit_usd),
            total_borrow_usd_scaled: parse_usd_scaled_6(&self.protocol_metrics.total_borrow_usd),
            loan_to_value_bps: self.protocol_metrics.loan_to_value_bps,
            liquidation_threshold_bps: self.protocol_metrics.liquidation_threshold_bps,
            borrow_utilization_bps: self.protocol_metrics.borrow_utilization_bps,
            net_account_value_usd_scaled: parse_usd_signed_scaled_6(
                &self.protocol_metrics.net_account_value_usd,
            ),
            is_protocol_reported: self.protocol_metrics.is_protocol_reported,
        };

        Ok(KaminoObligationSnapshot {
            obligation_id: self.obligation_id.clone(),
            owner: self.owner.clone(),
            market_id: self.market_id.clone(),
            observation_slot: self.observation_slot,
            observed_at: self.observed_at.clone(),
            protocol_source,
            deposits,
            borrows,
            protocol_metrics,
        })
    }

    pub fn to_position_snapshot(&self) -> Result<PositionSnapshot, ApiError> {
        let obl = self.to_domain()?;
        Ok(PositionSnapshot::from_kamino_obligation(&obl))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaminoObligationsListResponseJson {
    pub obligations: Vec<KaminoObligationResponseJson>,
}

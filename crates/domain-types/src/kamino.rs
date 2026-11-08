use serde::{Deserialize, Serialize};

use crate::amount::TokenAmount;
use crate::position::ProtocolSource;

/// Reserve configuration and state parameters observed from Kamino Lending.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaminoReserveSnapshot {
    pub reserve_id: String,
    pub liquidity_mint: String,
    pub collateral_mint: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub loan_to_value_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub liquidation_bonus_bps: u16,
    pub available_liquidity: TokenAmount,
    pub total_deposits: TokenAmount,
    pub total_borrows: TokenAmount,
    pub borrow_interest_rate_bps: u16,
    pub supply_interest_rate_bps: u16,
    pub oracle_address: String,
    pub last_update_slot: u64,
    pub status: String,
}

/// Lending market configuration observed from Kamino Lending.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaminoMarketSnapshot {
    pub market_id: String,
    pub owner: String,
    pub authority: String,
    pub program_id: String,
    pub network: String,
    pub observation_slot: u64,
    pub observed_at: String,
    pub reserves: Vec<KaminoReserveSnapshot>,
}

/// Individual collateral deposit in a Kamino obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaminoDepositSnapshot {
    pub reserve_address: String,
    pub mint_address: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub raw_amount: TokenAmount,
    pub market_value_usd_scaled: u128, // Scaled 6 decimals (micro-USD)
}

/// Individual debt borrow in a Kamino obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaminoBorrowSnapshot {
    pub reserve_address: String,
    pub mint_address: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub raw_amount: TokenAmount,
    pub market_value_usd_scaled: u128, // Scaled 6 decimals (micro-USD)
}

/// Protocol-reported official risk metrics from Kamino obligation calculations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaminoProtocolMetrics {
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

/// Normalized snapshot of a user obligation on Kamino Lending.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaminoObligationSnapshot {
    pub obligation_id: String,
    pub owner: String,
    pub market_id: String,
    pub observation_slot: u64,
    pub observed_at: String,
    pub protocol_source: ProtocolSource,
    pub deposits: Vec<KaminoDepositSnapshot>,
    pub borrows: Vec<KaminoBorrowSnapshot>,
    pub protocol_metrics: KaminoProtocolMetrics,
}

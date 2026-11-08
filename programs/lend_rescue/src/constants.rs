use anchor_lang::prelude::*;

#[constant]
pub const MARKET_SEED: &[u8] = b"market";

#[constant]
pub const ORACLE_SEED: &[u8] = b"oracle";

#[constant]
pub const POSITION_SEED: &[u8] = b"position";

#[constant]
pub const POLICY_SEED: &[u8] = b"policy";

#[constant]
pub const DEMO_POOL_SEED: &[u8] = b"demo_pool";

#[constant]
pub const RECEIPT_SEED: &[u8] = b"receipt";

#[constant]
pub const COLLATERAL_VAULT_SEED: &[u8] = b"collateral_vault";

#[constant]
pub const POOL_VAULT_SEED: &[u8] = b"pool_vault";

#[constant]
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";

/// Basis points divisor: 10,000 = 100%
pub const BPS_DIVISOR: u64 = 10_000;

/// Health factor fixed-point scale factor (1.0000 = 10,000)
pub const HF_PRECISION: u64 = 10_000;

/// Maximum allowed age of an oracle price update (60 seconds)
pub const MAX_ORACLE_AGE_SECS: i64 = 60;

/// Maximum allowable oracle confidence interval as basis points of price (200 bps = 2%)
pub const MAX_CONFIDENCE_BPS: u64 = 200;

/// Maximum acceptable slippage limit for policies (1,000 bps = 10%)
pub const MAX_PERMITTED_SLIPPAGE_BPS: u16 = 1_000;

/// Minimum health factor floor: cannot create position or policy below 1.0 (10,000 scaled)
pub const MIN_SAFE_HF: u64 = 10_000;

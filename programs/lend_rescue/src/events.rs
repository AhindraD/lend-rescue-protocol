use anchor_lang::prelude::*;

#[event]
pub struct MarketInitializedEvent {
    pub market: Pubkey,
    pub authority: Pubkey,
    pub collateral_mint: Pubkey,
    pub debt_mint: Pubkey,
    pub liquidation_threshold_bps: u16,
    pub timestamp: i64,
}

#[event]
pub struct MockOracleInitializedEvent {
    pub oracle: Pubkey,
    pub market: Pubkey,
    pub asset_mint: Pubkey,
    pub price: i64,
    pub confidence: u64,
    pub exponent: i32,
    pub timestamp: i64,
}

#[event]
pub struct MockOracleUpdatedEvent {
    pub oracle: Pubkey,
    pub market: Pubkey,
    pub asset_mint: Pubkey,
    pub old_price: i64,
    pub new_price: i64,
    pub confidence: u64,
    pub timestamp: i64,
}

#[event]
pub struct PositionCreatedEvent {
    pub position: Pubkey,
    pub market: Pubkey,
    pub owner: Pubkey,
    pub position_id: u64,
    pub timestamp: i64,
}

#[event]
pub struct CollateralDepositedEvent {
    pub position: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub total_collateral: u64,
    pub timestamp: i64,
}

#[event]
pub struct DemoDebtBorrowedEvent {
    pub position: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub total_debt: u64,
    pub timestamp: i64,
}

#[event]
pub struct DemoSwapPoolInitializedEvent {
    pub pool: Pubkey,
    pub market: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub fee_bps: u16,
    pub timestamp: i64,
}

#[event]
pub struct RecoveryPolicyCreatedEvent {
    pub policy: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub authorized_delegate: Pubkey,
    pub max_debt_repayment: u64,
    pub max_collateral_spend: u64,
    pub min_resulting_hf: u64,
    pub nonce: u64,
    pub expiry: i64,
    pub timestamp: i64,
}

#[event]
pub struct RecoveryPolicyUpdatedEvent {
    pub policy: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub authorized_delegate: Pubkey,
    pub new_nonce: u64,
    pub new_expiry: i64,
    pub timestamp: i64,
}

#[event]
pub struct RecoveryPolicyRevokedEvent {
    pub policy: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub nonce: u64,
    pub timestamp: i64,
}

#[event]
pub struct RecoveryExecutedEvent {
    pub receipt: Pubkey,
    pub position: Pubkey,
    pub policy: Pubkey,
    pub owner: Pubkey,
    pub caller: Pubkey,
    pub action: u8,
    pub input_amount: u64,
    pub output_amount: u64,
    pub pre_hf: u64,
    pub post_hf: u64,
    pub nonce_consumed: u64,
    pub slot: u64,
    pub timestamp: i64,
}

#[event]
pub struct MarketPausedEvent {
    pub market: Pubkey,
    pub authority: Pubkey,
    pub is_paused: bool,
    pub timestamp: i64,
}

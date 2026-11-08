use anchor_lang::prelude::*;

pub const ACTION_REPAY_DEBT: u8 = 1 << 0; // 1
pub const ACTION_ADD_COLLATERAL: u8 = 1 << 1; // 2
pub const ACTION_SWAP_AND_REPAY: u8 = 1 << 2; // 4
pub const ALL_ACTIONS_MASK: u8 = ACTION_REPAY_DEBT | ACTION_ADD_COLLATERAL | ACTION_SWAP_AND_REPAY;

#[account]
pub struct RecoveryPolicy {
    /// Associated lending position
    pub position: Pubkey,
    /// Owner of the position who authored this policy
    pub owner: Pubkey,
    /// Optional designated delegate key (or Pubkey::default() for owner-only)
    pub authorized_delegate: Pubkey,
    /// Maximum base units of debt token permitted for repayment
    pub max_debt_repayment: u64,
    /// Maximum base units of collateral token permitted for expenditure
    pub max_collateral_spend: u64,
    /// Maximum allowed slippage in basis points (e.g. 100 = 1%)
    pub max_slippage_bps: u16,
    /// Minimum acceptable post-recovery health factor scaled by HF_PRECISION (10_000 = 1.0)
    pub min_resulting_hf: u64,
    /// Permitted recovery action bitset
    pub allowed_actions: u8,
    /// Unix timestamp when this policy expires
    pub expiry: i64,
    /// Execution nonce (incremented on each successful recovery or update)
    pub nonce: u64,
    /// Active status switch
    pub is_active: bool,
    /// Schema version
    pub version: u8,
    /// PDA bump
    pub bump: u8,
}

impl RecoveryPolicy {
    pub const LEN: usize = 8 // discriminator
        + 32 // position
        + 32 // owner
        + 32 // authorized_delegate
        + 8  // max_debt_repayment
        + 8  // max_collateral_spend
        + 2  // max_slippage_bps
        + 8  // min_resulting_hf
        + 1  // allowed_actions
        + 8  // expiry
        + 8  // nonce
        + 1  // is_active
        + 1  // version
        + 1  // bump
        + 32; // reserved padding

    pub fn is_action_allowed(&self, action: u8) -> bool {
        (self.allowed_actions & action) != 0
    }
}


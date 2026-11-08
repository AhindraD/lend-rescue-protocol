use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct RecoveryReceipt {
    /// Position that was rescued
    pub position: Pubkey,
    /// Owner of the position
    pub owner: Pubkey,
    /// Recovery policy governing the execution
    pub policy: Pubkey,
    /// Actual signer who initiated execution (owner or authorized delegate)
    pub delegate: Pubkey,
    /// Action executed (1 = Repay, 2 = AddCollateral, 4 = SwapAndRepay)
    pub action_executed: u8,
    /// Base units of input token expended
    pub input_amount: u64,
    /// Base units of output token produced (for swaps)
    pub output_amount: u64,
    /// Base units of debt liability extinguished
    pub repayment_amount: u64,
    /// Health factor immediately prior to execution
    pub pre_health_factor: u64,
    /// Health factor derived immediately following execution
    pub post_health_factor: u64,
    /// Policy nonce that was consumed
    pub nonce_consumed: u64,
    /// Solana slot of execution
    pub execution_slot: u64,
    /// Unix timestamp of execution
    pub execution_timestamp: i64,
    /// Status code (0 = Success)
    pub status: u8,
    /// Schema version
    pub version: u8,
    /// PDA bump
    pub bump: u8,
}

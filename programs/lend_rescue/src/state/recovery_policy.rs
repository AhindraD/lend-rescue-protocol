use anchor_lang::prelude::*;

pub const ACTION_REPAY_DEBT: u8 = 1 << 0; // 1
pub const ACTION_ADD_COLLATERAL: u8 = 1 << 1; // 2
pub const ACTION_SWAP_AND_REPAY: u8 = 1 << 2; // 4
pub const ALL_ACTIONS_MASK: u8 = ACTION_REPAY_DEBT | ACTION_ADD_COLLATERAL | ACTION_SWAP_AND_REPAY;

pub const RECOVERY_POLICY_SCHEMA_VERSION: u8 = 2;

#[account]
#[derive(InitSpace)]
pub struct RecoveryPolicy {
    /// Associated lending position
    pub position: Pubkey,

    /// Owner of the lending position and policy authority
    pub owner: Pubkey,

    /// Optional authorized recovery executor.
    /// None means owner-only execution.
    pub authorized_delegate: Option<Pubkey>,

    /// Maximum debt-token base units permitted for repayment
    pub max_debt_repayment: u64,

    /// Maximum collateral-token base units permitted for expenditure
    pub max_collateral_spend: u64,

    /// Maximum permitted slippage in basis points (e.g. 100 = 1%)
    pub max_slippage_bps: u16,

    /// Minimum acceptable post-recovery health factor scaled by HF_PRECISION (10_000 = 1.0)
    pub min_resulting_hf: u64,

    /// Permitted recovery action bitset
    pub allowed_actions: u8,

    /// Unix timestamp when the policy expires
    pub expiry: i64,

    /// Replay protection for recovery executions (starts at 0, incremented only on successful recovery)
    pub execution_nonce: u64,

    /// Policy configuration revision (starts at 1, incremented on updates and revocation)
    pub policy_revision: u64,

    /// Whether the policy is currently active
    pub is_active: bool,

    /// Serialized account layout version
    pub schema_version: u8,

    /// PDA bump
    pub bump: u8,
}

impl RecoveryPolicy {
    /// Checks if a specific recovery action is permitted by this policy.
    pub fn is_action_allowed(&self, action: u8) -> bool {
        (self.allowed_actions & action) != 0
    }

    /// Checks if the given caller is authorized to execute recovery under this policy.
    /// Either the position owner or the designated delegate is authorized.
    pub fn is_authorized_caller(&self, caller: &Pubkey) -> bool {
        if &self.owner == caller {
            return true;
        }
        if let Some(ref delegate) = self.authorized_delegate {
            return delegate == caller;
        }
        false
    }
}

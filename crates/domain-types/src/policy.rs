use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::recovery::RecoveryAction;

pub const ACTION_REPAY_DEBT: u8 = 1;
pub const ACTION_ADD_COLLATERAL: u8 = 2;
pub const ACTION_SWAP_AND_REPAY: u8 = 4;
pub const ALL_ACTIONS_MASK: u8 = ACTION_REPAY_DEBT | ACTION_ADD_COLLATERAL | ACTION_SWAP_AND_REPAY;

/// Off-chain and domain mirror of the on-chain Anchor `RecoveryPolicy`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryPolicy {
    /// Associated lending position ID or pubkey string
    pub position: String,
    /// Owner of the position who authored this policy
    pub owner: String,
    /// Designated delegate key (or None / empty for owner-only)
    pub authorized_delegate: Option<String>,
    /// Maximum base units of debt token permitted for repayment
    pub max_debt_repayment: u64,
    /// Maximum base units of collateral token permitted for expenditure
    pub max_collateral_spend: u64,
    /// Maximum allowed slippage in basis points (e.g. 100 = 1%)
    pub max_slippage_bps: u16,
    /// Minimum acceptable post-recovery health factor scaled by 10,000 (10,000 = 1.0)
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
}

impl RecoveryPolicy {
    /// Checks if a specific recovery action is permitted by this policy.
    pub fn is_action_allowed(&self, action: RecoveryAction) -> bool {
        (self.allowed_actions & action.bitmask()) != 0
    }

    /// Checks if a given signer is authorized to execute this policy.
    /// Either the position owner or the designated delegate is authorized.
    pub fn is_authorized_caller(&self, signer: &str) -> bool {
        if self.owner == signer {
            return true;
        }
        if let Some(ref delegate) = self.authorized_delegate {
            if !delegate.is_empty() && delegate == signer {
                return true;
            }
        }
        false
    }

    /// Checks if this policy is expired relative to current unix timestamp.
    pub fn is_expired(&self, current_timestamp: i64) -> bool {
        current_timestamp >= self.expiry
    }

    /// Validates policy bounds against protocol constraints.
    pub fn validate_bounds(&self) -> Result<(), DomainError> {
        if self.allowed_actions == 0 || (self.allowed_actions & !ALL_ACTIONS_MASK) != 0 {
            return Err(DomainError::InvalidPolicy(
                "Allowed actions must contain at least one valid action bit",
            ));
        }
        if self.max_debt_repayment == 0
            && (self.allowed_actions & (ACTION_REPAY_DEBT | ACTION_SWAP_AND_REPAY)) != 0
        {
            return Err(DomainError::InvalidPolicy(
                "max_debt_repayment must be > 0 when repayment actions are enabled",
            ));
        }
        if self.max_collateral_spend == 0
            && (self.allowed_actions & (ACTION_ADD_COLLATERAL | ACTION_SWAP_AND_REPAY)) != 0
        {
            return Err(DomainError::InvalidPolicy(
                "max_collateral_spend must be > 0 when collateral actions are enabled",
            ));
        }
        if self.max_slippage_bps > 1_000 {
            return Err(DomainError::InvalidPolicy(
                "max_slippage_bps cannot exceed 1000 bps (10%)",
            ));
        }
        if self.min_resulting_hf < 10_000 {
            return Err(DomainError::InvalidPolicy(
                "min_resulting_hf must be at least 1.0 (10,000 scaled)",
            ));
        }
        Ok(())
    }
}

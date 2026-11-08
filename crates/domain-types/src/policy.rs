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
    /// Designated delegate key (or None for owner-only)
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
    /// Execution nonce for replay protection (starts at 0, incremented only on successful recovery)
    pub execution_nonce: u64,
    /// Policy configuration revision (starts at 1, incremented on updates and revocation)
    pub policy_revision: u64,
    /// Active status switch
    pub is_active: bool,
    /// Schema version
    pub schema_version: u8,
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
        current_timestamp > self.expiry
    }

    /// Returns the execution replay nonce.
    pub fn nonce(&self) -> u64 {
        self.execution_nonce
    }

    /// Returns the schema layout version.
    pub fn version(&self) -> u8 {
        self.schema_version
    }

    /// Validates basic policy invariant bounds.
    pub fn validate_bounds(&self) -> Result<(), DomainError> {
        if self.max_debt_repayment == 0 || self.max_collateral_spend == 0 {
            return Err(DomainError::InvalidPolicy(
                "Spending limits must be non-zero",
            ));
        }
        if self.max_slippage_bps > 1_000 {
            return Err(DomainError::InvalidPolicy(
                "Slippage exceeds 1000 bps maximum",
            ));
        }
        if self.min_resulting_hf < 10_000 {
            return Err(DomainError::InvalidPolicy(
                "Minimum resulting HF must be at least 1.0",
            ));
        }
        if (self.allowed_actions & ALL_ACTIONS_MASK) == 0 {
            return Err(DomainError::InvalidPolicy(
                "At least one recovery action must be permitted",
            ));
        }
        Ok(())
    }
}

use anchor_lang::prelude::*;

use crate::error::LendRescueError;
use crate::events::RecoveryPolicyRevokedEvent;
use crate::state::RecoveryPolicy;

#[derive(Accounts)]
pub struct RevokeRecoveryPolicy<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = owner @ LendRescueError::InvalidPolicyOwner,
        constraint = policy.is_active @ LendRescueError::PolicyRevoked
    )]
    pub policy: Account<'info, RecoveryPolicy>,
}

impl<'info> RevokeRecoveryPolicy<'info> {
    pub fn revoke_recovery_policy(&mut self) -> Result<()> {
        let policy = &mut self.policy;
        policy.is_active = false;

        // Increment policy revision to track configuration state change (execution_nonce is untouched)
        policy.policy_revision = policy
            .policy_revision
            .checked_add(1)
            .ok_or(LendRescueError::MathOverflow)?;

        emit!(RecoveryPolicyRevokedEvent {
            policy: policy.key(),
            position: policy.position,
            owner: policy.owner,
            policy_revision: policy.policy_revision,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

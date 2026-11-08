use anchor_lang::prelude::*;

use crate::error::LendRescueError;
use crate::events::RecoveryPolicyRevokedEvent;
use crate::state::RecoveryPolicy;

#[derive(Accounts)]
pub struct RevokeRecoveryPolicy<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = owner @ LendRescueError::InvalidPolicyOwner
    )]
    pub policy: Account<'info, RecoveryPolicy>,
}

pub fn handle_revoke_recovery_policy(ctx: Context<RevokeRecoveryPolicy>) -> Result<()> {
    let policy = &mut ctx.accounts.policy;
    policy.is_active = false;

    policy.nonce = policy
        .nonce
        .checked_add(1)
        .ok_or(LendRescueError::MathOverflow)?;

    emit!(RecoveryPolicyRevokedEvent {
        policy: policy.key(),
        position: policy.position,
        owner: policy.owner,
        nonce: policy.nonce,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}


use anchor_lang::prelude::*;

use crate::constants::{MAX_PERMITTED_SLIPPAGE_BPS, MIN_SAFE_HF};
use crate::error::LendRescueError;
use crate::events::RecoveryPolicyUpdatedEvent;
use crate::state::{RecoveryPolicy, ALL_ACTIONS_MASK};

#[derive(Accounts)]
pub struct UpdateRecoveryPolicy<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = owner @ LendRescueError::InvalidPolicyOwner
    )]
    pub policy: Account<'info, RecoveryPolicy>,
}

#[allow(clippy::too_many_arguments)]
pub fn handle_update_recovery_policy(
    ctx: Context<UpdateRecoveryPolicy>,
    new_delegate: Pubkey,
    new_max_debt_repayment: u64,
    new_max_collateral_spend: u64,
    new_max_slippage_bps: u16,
    new_min_resulting_hf: u64,
    new_allowed_actions: u8,
    new_expiry: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(new_expiry > now, LendRescueError::PolicyExpired);
    require!(
        new_max_debt_repayment > 0,
        LendRescueError::InvalidPolicyBounds
    );
    require!(
        new_max_collateral_spend > 0,
        LendRescueError::InvalidPolicyBounds
    );
    require!(
        new_max_slippage_bps <= MAX_PERMITTED_SLIPPAGE_BPS,
        LendRescueError::SlippageExceeded
    );
    require!(
        new_min_resulting_hf >= MIN_SAFE_HF,
        LendRescueError::TargetHealthFactorTooLow
    );
    require!(
        (new_allowed_actions & ALL_ACTIONS_MASK) != 0,
        LendRescueError::InvalidAllowedActions
    );

    let policy = &mut ctx.accounts.policy;
    policy.authorized_delegate = new_delegate;
    policy.max_debt_repayment = new_max_debt_repayment;
    policy.max_collateral_spend = new_max_collateral_spend;
    policy.max_slippage_bps = new_max_slippage_bps;
    policy.min_resulting_hf = new_min_resulting_hf;
    policy.allowed_actions = new_allowed_actions;
    policy.expiry = new_expiry;
    policy.is_active = true;

    // Increment nonce to safely invalidate any prior signed plans
    policy.nonce = policy
        .nonce
        .checked_add(1)
        .ok_or(LendRescueError::MathOverflow)?;

    emit!(RecoveryPolicyUpdatedEvent {
        policy: policy.key(),
        position: policy.position,
        owner: policy.owner,
        authorized_delegate: new_delegate,
        new_nonce: policy.nonce,
        new_expiry,
        timestamp: now,
    });

    Ok(())
}

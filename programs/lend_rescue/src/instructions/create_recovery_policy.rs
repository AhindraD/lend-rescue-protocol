use anchor_lang::prelude::*;

use crate::constants::{MAX_PERMITTED_SLIPPAGE_BPS, MIN_SAFE_HF, POLICY_SEED};
use crate::error::LendRescueError;
use crate::events::RecoveryPolicyCreatedEvent;
use crate::state::{Position, RecoveryPolicy, ALL_ACTIONS_MASK};

#[derive(Accounts)]
pub struct CreateRecoveryPolicy<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        has_one = owner @ LendRescueError::InvalidPositionOwner
    )]
    pub position: Account<'info, Position>,

    #[account(
        init,
        payer = owner,
        space = RecoveryPolicy::LEN,
        seeds = [POLICY_SEED, position.key().as_ref()],
        bump
    )]
    pub policy: Account<'info, RecoveryPolicy>,

    pub system_program: Program<'info, System>,
}

#[allow(clippy::too_many_arguments)]
pub fn handle_create_recovery_policy(
    ctx: Context<CreateRecoveryPolicy>,
    authorized_delegate: Pubkey,
    max_debt_repayment: u64,
    max_collateral_spend: u64,
    max_slippage_bps: u16,
    min_resulting_hf: u64,
    allowed_actions: u8,
    expiry: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(expiry > now, LendRescueError::PolicyExpired);
    require!(max_debt_repayment > 0, LendRescueError::InvalidPolicyBounds);
    require!(
        max_collateral_spend > 0,
        LendRescueError::InvalidPolicyBounds
    );
    require!(
        max_slippage_bps <= MAX_PERMITTED_SLIPPAGE_BPS,
        LendRescueError::SlippageExceeded
    );
    require!(
        min_resulting_hf >= MIN_SAFE_HF,
        LendRescueError::TargetHealthFactorTooLow
    );
    require!(
        (allowed_actions & ALL_ACTIONS_MASK) != 0,
        LendRescueError::InvalidAllowedActions
    );

    let policy = &mut ctx.accounts.policy;
    policy.position = ctx.accounts.position.key();
    policy.owner = ctx.accounts.owner.key();
    policy.authorized_delegate = authorized_delegate;
    policy.max_debt_repayment = max_debt_repayment;
    policy.max_collateral_spend = max_collateral_spend;
    policy.max_slippage_bps = max_slippage_bps;
    policy.min_resulting_hf = min_resulting_hf;
    policy.allowed_actions = allowed_actions;
    policy.expiry = expiry;
    policy.nonce = 1;
    policy.is_active = true;
    policy.version = 1;
    policy.bump = ctx.bumps.policy;

    emit!(RecoveryPolicyCreatedEvent {
        policy: policy.key(),
        position: policy.position,
        owner: policy.owner,
        authorized_delegate,
        max_debt_repayment,
        max_collateral_spend,
        min_resulting_hf,
        nonce: policy.nonce,
        expiry,
        timestamp: now,
    });

    Ok(())
}

use anchor_lang::prelude::*;

use crate::constants::{MAX_PERMITTED_SLIPPAGE_BPS, MIN_SAFE_HF};
use crate::error::LendRescueError;
use crate::events::RecoveryPolicyUpdatedEvent;
use crate::state::{RecoveryPolicy, ALL_ACTIONS_MASK};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UpdateRecoveryPolicyArgs {
    pub new_delegate: Option<Pubkey>,
    pub new_max_debt_repayment: u64,
    pub new_max_collateral_spend: u64,
    pub new_max_slippage_bps: u16,
    pub new_min_resulting_hf: u64,
    pub new_allowed_actions: u8,
    pub new_expiry: i64,
}

#[derive(Accounts)]
pub struct UpdateRecoveryPolicy<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = owner @ LendRescueError::InvalidPolicyOwner,
        constraint = policy.is_active @ LendRescueError::PolicyRevoked
    )]
    pub policy: Account<'info, RecoveryPolicy>,
}

impl<'info> UpdateRecoveryPolicy<'info> {
    pub fn update_recovery_policy(&mut self, args: UpdateRecoveryPolicyArgs) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(args.new_expiry > now, LendRescueError::PolicyExpired);
        require!(
            args.new_max_debt_repayment > 0,
            LendRescueError::InvalidPolicyBounds
        );
        require!(
            args.new_max_collateral_spend > 0,
            LendRescueError::InvalidPolicyBounds
        );
        require!(
            args.new_max_slippage_bps <= MAX_PERMITTED_SLIPPAGE_BPS,
            LendRescueError::SlippageExceeded
        );
        require!(
            args.new_min_resulting_hf >= MIN_SAFE_HF,
            LendRescueError::TargetHealthFactorTooLow
        );
        require!(
            (args.new_allowed_actions & ALL_ACTIONS_MASK) != 0,
            LendRescueError::InvalidAllowedActions
        );

        let policy = &mut self.policy;
        policy.authorized_delegate = args.new_delegate;
        policy.max_debt_repayment = args.new_max_debt_repayment;
        policy.max_collateral_spend = args.new_max_collateral_spend;
        policy.max_slippage_bps = args.new_max_slippage_bps;
        policy.min_resulting_hf = args.new_min_resulting_hf;
        policy.allowed_actions = args.new_allowed_actions;
        policy.expiry = args.new_expiry;
        policy.is_active = true;

        // Increment policy revision to track configuration updates (execution_nonce is untouched)
        policy.policy_revision = policy
            .policy_revision
            .checked_add(1)
            .ok_or(LendRescueError::MathOverflow)?;

        emit!(RecoveryPolicyUpdatedEvent {
            policy: policy.key(),
            position: policy.position,
            owner: policy.owner,
            authorized_delegate: args.new_delegate,
            policy_revision: policy.policy_revision,
            new_expiry: args.new_expiry,
            timestamp: now,
        });

        Ok(())
    }
}

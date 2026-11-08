use anchor_lang::prelude::*;

use crate::constants::{MAX_PERMITTED_SLIPPAGE_BPS, MIN_SAFE_HF, POLICY_SEED};
use crate::error::LendRescueError;
use crate::events::RecoveryPolicyCreatedEvent;
use crate::state::{Position, RecoveryPolicy, ALL_ACTIONS_MASK, RECOVERY_POLICY_SCHEMA_VERSION};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreateRecoveryPolicyArgs {
    pub authorized_delegate: Option<Pubkey>,
    pub max_debt_repayment: u64,
    pub max_collateral_spend: u64,
    pub max_slippage_bps: u16,
    pub min_resulting_hf: u64,
    pub allowed_actions: u8,
    pub expiry: i64,
}

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
        space = 8 + RecoveryPolicy::INIT_SPACE,
        seeds = [POLICY_SEED, position.key().as_ref()],
        bump
    )]
    pub policy: Account<'info, RecoveryPolicy>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateRecoveryPolicy<'info> {
    pub fn create_recovery_policy(
        &mut self,
        args: CreateRecoveryPolicyArgs,
        bump: u8,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(args.expiry > now, LendRescueError::PolicyExpired);
        require!(
            args.max_debt_repayment > 0,
            LendRescueError::InvalidPolicyBounds
        );
        require!(
            args.max_collateral_spend > 0,
            LendRescueError::InvalidPolicyBounds
        );
        require!(
            args.max_slippage_bps <= MAX_PERMITTED_SLIPPAGE_BPS,
            LendRescueError::SlippageExceeded
        );
        require!(
            args.min_resulting_hf >= MIN_SAFE_HF,
            LendRescueError::TargetHealthFactorTooLow
        );
        require!(
            (args.allowed_actions & ALL_ACTIONS_MASK) != 0,
            LendRescueError::InvalidAllowedActions
        );

        let policy = &mut self.policy;
        policy.position = self.position.key();
        policy.owner = self.owner.key();
        policy.authorized_delegate = args.authorized_delegate;
        policy.max_debt_repayment = args.max_debt_repayment;
        policy.max_collateral_spend = args.max_collateral_spend;
        policy.max_slippage_bps = args.max_slippage_bps;
        policy.min_resulting_hf = args.min_resulting_hf;
        policy.allowed_actions = args.allowed_actions;
        policy.expiry = args.expiry;
        policy.execution_nonce = 0; // Execution nonce starts at zero for replay protection
        policy.policy_revision = 1; // Policy configuration revision starts at one
        policy.is_active = true;
        policy.schema_version = RECOVERY_POLICY_SCHEMA_VERSION;
        policy.bump = bump;

        emit!(RecoveryPolicyCreatedEvent {
            policy: policy.key(),
            position: policy.position,
            owner: policy.owner,
            authorized_delegate: args.authorized_delegate,
            max_debt_repayment: args.max_debt_repayment,
            max_collateral_spend: args.max_collateral_spend,
            min_resulting_hf: args.min_resulting_hf,
            execution_nonce: policy.execution_nonce,
            policy_revision: policy.policy_revision,
            expiry: args.expiry,
            timestamp: now,
        });

        Ok(())
    }
}

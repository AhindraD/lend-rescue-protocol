#![allow(unexpected_cfgs)]
#![allow(clippy::too_many_arguments)]

use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("FDH6cF9xvoTBpJiPgAEciVcNueEqhLYxXQcPDpM6PpUV");

#[program]
pub mod lend_rescue {
    use super::*;

    /// Initializes a synthetic lending market with collateral and debt mint parameters
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        market_id: u64,
        liquidation_threshold_bps: u16,
        demo_swap_fee_bps: u16,
    ) -> Result<()> {
        instructions::initialize_market::handle_initialize_market(
            ctx,
            market_id,
            liquidation_threshold_bps,
            demo_swap_fee_bps,
        )
    }

    /// Initializes a mock oracle feed for an asset in this market
    pub fn initialize_mock_oracle(
        ctx: Context<InitializeMockOracle>,
        initial_price: i64,
        exponent: i32,
        confidence: u64,
    ) -> Result<()> {
        instructions::initialize_mock_oracle::handle_initialize_mock_oracle(
            ctx,
            initial_price,
            exponent,
            confidence,
        )
    }

    /// Updates the mock oracle price mantissa and confidence (update authority only)
    pub fn update_mock_price(
        ctx: Context<UpdateMockPrice>,
        new_price: i64,
        confidence: u64,
    ) -> Result<()> {
        instructions::update_mock_price::handle_update_mock_price(ctx, new_price, confidence)
    }

    /// Creates a user lending position in the synthetic market
    pub fn create_demo_position(ctx: Context<CreateDemoPosition>, position_id: u64) -> Result<()> {
        instructions::create_demo_position::handle_create_demo_position(ctx, position_id)
    }

    /// Deposits collateral tokens into the market collateral vault
    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        instructions::deposit_collateral::handle_deposit_collateral(ctx, amount)
    }

    /// Mints synthetic debt tokens against deposited collateral
    pub fn borrow_demo_debt(ctx: Context<BorrowDemoDebt>, amount: u64) -> Result<()> {
        instructions::borrow_demo_debt::handle_borrow_demo_debt(ctx, amount)
    }

    /// Initializes a deterministic demo swap pool for synthetic route execution
    pub fn initialize_demo_swap_pool(
        ctx: Context<InitializeDemoSwapPool>,
        fee_bps: u16,
    ) -> Result<()> {
        instructions::initialize_demo_swap_pool::handle_initialize_demo_swap_pool(ctx, fee_bps)
    }

    /// Creates an on-chain recovery policy constraining future rescue transactions
    #[allow(clippy::too_many_arguments)]
    pub fn create_recovery_policy(
        ctx: Context<CreateRecoveryPolicy>,
        authorized_delegate: Pubkey,
        max_debt_repayment: u64,
        max_collateral_spend: u64,
        max_slippage_bps: u16,
        min_resulting_hf: u64,
        allowed_actions: u8,
        expiry: i64,
    ) -> Result<()> {
        instructions::create_recovery_policy::handle_create_recovery_policy(
            ctx,
            authorized_delegate,
            max_debt_repayment,
            max_collateral_spend,
            max_slippage_bps,
            min_resulting_hf,
            allowed_actions,
            expiry,
        )
    }

    /// Updates recovery policy parameters and increments the execution nonce
    #[allow(clippy::too_many_arguments)]
    pub fn update_recovery_policy(
        ctx: Context<UpdateRecoveryPolicy>,
        new_delegate: Pubkey,
        new_max_debt_repayment: u64,
        new_max_collateral_spend: u64,
        new_max_slippage_bps: u16,
        new_min_resulting_hf: u64,
        new_allowed_actions: u8,
        new_expiry: i64,
    ) -> Result<()> {
        instructions::update_recovery_policy::handle_update_recovery_policy(
            ctx,
            new_delegate,
            new_max_debt_repayment,
            new_max_collateral_spend,
            new_max_slippage_bps,
            new_min_resulting_hf,
            new_allowed_actions,
            new_expiry,
        )
    }

    /// Immediately revokes a recovery policy and increments the execution nonce
    pub fn revoke_recovery_policy(ctx: Context<RevokeRecoveryPolicy>) -> Result<()> {
        instructions::revoke_recovery_policy::handle_revoke_recovery_policy(ctx)
    }

    /// Executes an authorized recovery action within strict on-chain policy bounds
    pub fn execute_recovery(
        ctx: Context<ExecuteRecovery>,
        args: ExecuteRecoveryArgs,
    ) -> Result<()> {
        instructions::execute_recovery::handle_execute_recovery(ctx, args)
    }

    /// Emergency pause/unpause switch for market authority
    pub fn pause_market(ctx: Context<PauseMarket>, is_paused: bool) -> Result<()> {
        instructions::pause_market::handle_pause_market(ctx, is_paused)
    }
}

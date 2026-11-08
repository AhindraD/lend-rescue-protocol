use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::constants::{DEMO_POOL_SEED, MARKET_SEED, MAX_ORACLE_AGE_SECS, RECEIPT_SEED};
use crate::error::LendRescueError;
use crate::events::RecoveryExecutedEvent;
use crate::state::{
    DemoSwapPool, MarketConfig, MockOracle, Position, RecoveryPolicy, RecoveryReceipt,
    ACTION_ADD_COLLATERAL, ACTION_REPAY_DEBT, ACTION_SWAP_AND_REPAY,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct ExecuteRecoveryArgs {
    pub action: u8,
    pub input_amount: u64,
    pub min_output_amount: u64,
    pub nonce: u64,
}

#[derive(Accounts)]
#[instruction(args: ExecuteRecoveryArgs)]
pub struct ExecuteRecovery<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        constraint = !market.is_paused @ LendRescueError::MarketPaused
    )]
    pub market: Box<Account<'info, MarketConfig>>,

    #[account(
        mut,
        has_one = market,
        constraint = position.owner == policy.owner @ LendRescueError::InvalidPositionOwner
    )]
    pub position: Box<Account<'info, Position>>,

    #[account(
        mut,
        has_one = position,
        constraint = policy.is_active @ LendRescueError::PolicyRevoked
    )]
    pub policy: Box<Account<'info, RecoveryPolicy>>,

    #[account(
        constraint = collateral_oracle.key() == market.collateral_oracle @ LendRescueError::OracleAssetMismatch
    )]
    pub collateral_oracle: Box<Account<'info, MockOracle>>,

    #[account(
        constraint = debt_oracle.key() == market.debt_oracle @ LendRescueError::OracleAssetMismatch
    )]
    pub debt_oracle: Box<Account<'info, MockOracle>>,

    #[account(
        mut,
        constraint = collateral_vault.key() == market.collateral_vault @ LendRescueError::InvalidMint
    )]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = debt_mint.key() == market.debt_mint @ LendRescueError::InvalidMint
    )]
    pub debt_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = user_collateral_account.mint == market.collateral_mint @ LendRescueError::InvalidMint
    )]
    pub user_collateral_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_debt_account.mint == market.debt_mint @ LendRescueError::InvalidMint
    )]
    pub user_debt_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            DEMO_POOL_SEED,
            market.key().as_ref(),
            market.collateral_mint.as_ref(),
            market.debt_mint.as_ref()
        ],
        bump = demo_pool.bump
    )]
    pub demo_pool: Box<Account<'info, DemoSwapPool>>,

    #[account(
        mut,
        constraint = pool_input_vault.key() == demo_pool.input_vault @ LendRescueError::InvalidMint
    )]
    pub pool_input_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = pool_output_vault.key() == demo_pool.output_vault @ LendRescueError::InvalidMint
    )]
    pub pool_output_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = authority,
        space = RecoveryReceipt::LEN,
        seeds = [RECEIPT_SEED, position.key().as_ref(), &policy.nonce.to_le_bytes()],
        bump
    )]
    pub receipt: Box<Account<'info, RecoveryReceipt>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[inline(never)]
fn execute_repay_debt(
    ctx: &mut Context<ExecuteRecovery>,
    input_amount: u64,
    caller_key: Pubkey,
) -> Result<u64> {
    let debt_repaid = input_amount;
    require!(
        debt_repaid <= ctx.accounts.policy.max_debt_repayment,
        LendRescueError::ExceedsMaxDebtRepayment
    );

    require!(
        ctx.accounts.user_debt_account.owner == caller_key
            || ctx.accounts.user_debt_account.owner == ctx.accounts.policy.owner,
        LendRescueError::UnauthorizedCaller
    );

    let cpi_accounts = Burn {
        mint: ctx.accounts.debt_mint.to_account_info(),
        from: ctx.accounts.user_debt_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);
    token::burn(cpi_ctx, debt_repaid)?;

    let pos = &mut ctx.accounts.position;
    pos.debt_amount = pos.debt_amount.saturating_sub(debt_repaid);

    Ok(debt_repaid)
}

#[inline(never)]
fn execute_add_collateral(
    ctx: &mut Context<ExecuteRecovery>,
    input_amount: u64,
    caller_key: Pubkey,
) -> Result<()> {
    let collateral_added = input_amount;
    require!(
        collateral_added <= ctx.accounts.policy.max_collateral_spend,
        LendRescueError::ExceedsMaxCollateralSpend
    );

    require!(
        ctx.accounts.user_collateral_account.owner == caller_key
            || ctx.accounts.user_collateral_account.owner == ctx.accounts.policy.owner,
        LendRescueError::UnauthorizedCaller
    );

    let cpi_accounts = Transfer {
        from: ctx.accounts.user_collateral_account.to_account_info(),
        to: ctx.accounts.collateral_vault.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);
    token::transfer(cpi_ctx, collateral_added)?;

    let pos = &mut ctx.accounts.position;
    pos.collateral_amount = pos
        .collateral_amount
        .checked_add(collateral_added)
        .ok_or(LendRescueError::MathOverflow)?;

    Ok(())
}

#[inline(never)]
fn execute_swap_and_repay(
    ctx: &mut Context<ExecuteRecovery>,
    collateral_to_spend: u64,
    min_output_amount: u64,
) -> Result<(u64, u64)> {
    require!(
        collateral_to_spend <= ctx.accounts.policy.max_collateral_spend,
        LendRescueError::ExceedsMaxCollateralSpend
    );
    require!(
        ctx.accounts.position.collateral_amount >= collateral_to_spend,
        LendRescueError::InsufficientCollateral
    );

    let pool = &ctx.accounts.demo_pool;
    let (amount_out, _fee) = DemoSwapPool::calculate_swap_output(
        ctx.accounts.pool_input_vault.amount,
        ctx.accounts.pool_output_vault.amount,
        collateral_to_spend,
        pool.fee_bps,
    )?;

    require!(
        amount_out >= min_output_amount,
        LendRescueError::SlippageExceeded
    );

    let debt_repaid = amount_out;
    require!(
        debt_repaid <= ctx.accounts.policy.max_debt_repayment,
        LendRescueError::ExceedsMaxDebtRepayment
    );

    // Transfer collateral from collateral_vault to pool_input_vault using market PDA signer
    let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
    let market_seeds = &[
        MARKET_SEED,
        market_id_bytes.as_ref(),
        &[ctx.accounts.market.bump],
    ];
    let market_signer = &[&market_seeds[..]];

    let cpi_transfer_in = Transfer {
        from: ctx.accounts.collateral_vault.to_account_info(),
        to: ctx.accounts.pool_input_vault.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };
    let cpi_ctx_in = CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        cpi_transfer_in,
        market_signer,
    );
    token::transfer(cpi_ctx_in, collateral_to_spend)?;

    // Transfer output debt tokens from pool_output_vault directly to burning/repayment
    let market_key = ctx.accounts.market.key();
    let pool_seeds = &[
        DEMO_POOL_SEED,
        market_key.as_ref(),
        ctx.accounts.market.collateral_mint.as_ref(),
        ctx.accounts.market.debt_mint.as_ref(),
        &[ctx.accounts.demo_pool.bump],
    ];
    let pool_signer = &[&pool_seeds[..]];

    let cpi_burn_out = Burn {
        mint: ctx.accounts.debt_mint.to_account_info(),
        from: ctx.accounts.pool_output_vault.to_account_info(),
        authority: ctx.accounts.demo_pool.to_account_info(),
    };
    let cpi_ctx_burn =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi_burn_out, pool_signer);
    token::burn(cpi_ctx_burn, debt_repaid)?;

    let pos = &mut ctx.accounts.position;
    pos.collateral_amount = pos
        .collateral_amount
        .checked_sub(collateral_to_spend)
        .ok_or(LendRescueError::MathOverflow)?;
    pos.debt_amount = pos.debt_amount.saturating_sub(debt_repaid);

    Ok((amount_out, debt_repaid))
}

pub fn handle_execute_recovery(
    mut ctx: Context<ExecuteRecovery>,
    args: ExecuteRecoveryArgs,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let slot = Clock::get()?.slot;

    // 1. Authorization check: caller must be position owner OR authorized active delegate
    let caller_key = ctx.accounts.authority.key();
    let is_owner = caller_key == ctx.accounts.policy.owner;
    let is_delegate = ctx.accounts.policy.authorized_delegate != Pubkey::default()
        && caller_key == ctx.accounts.policy.authorized_delegate;
    require!(is_owner || is_delegate, LendRescueError::UnauthorizedCaller);

    // 2. Policy expiry and nonce verification
    require!(
        now <= ctx.accounts.policy.expiry,
        LendRescueError::PolicyExpired
    );
    require!(
        args.nonce == ctx.accounts.policy.nonce,
        LendRescueError::InvalidNonce
    );

    // 3. Action permission check
    require!(
        ctx.accounts.policy.is_action_allowed(args.action),
        LendRescueError::ActionNotAllowed
    );
    require!(args.input_amount > 0, LendRescueError::ZeroAmount);

    // 4. Oracle freshness and confidence validation
    require!(
        ctx.accounts
            .collateral_oracle
            .is_fresh(now, MAX_ORACLE_AGE_SECS),
        LendRescueError::StaleOracle
    );
    require!(
        ctx.accounts.debt_oracle.is_fresh(now, MAX_ORACLE_AGE_SECS),
        LendRescueError::StaleOracle
    );

    // 5. Compute pre-recovery health factor on-chain
    let market = &ctx.accounts.market;
    let position = &ctx.accounts.position;
    let pre_hf = Position::compute_synthetic_hf(
        position.collateral_amount,
        ctx.accounts.collateral_oracle.price,
        ctx.accounts.collateral_oracle.exponent,
        market.collateral_decimals,
        position.debt_amount,
        ctx.accounts.debt_oracle.price,
        ctx.accounts.debt_oracle.exponent,
        market.debt_decimals,
        market.liquidation_threshold_bps,
    )?;

    let actual_input = args.input_amount;
    let mut actual_output: u64 = 0;
    let mut debt_repaid: u64 = 0;

    match args.action {
        ACTION_REPAY_DEBT => {
            debt_repaid = execute_repay_debt(&mut ctx, args.input_amount, caller_key)?;
        }
        ACTION_ADD_COLLATERAL => {
            execute_add_collateral(&mut ctx, args.input_amount, caller_key)?;
        }
        ACTION_SWAP_AND_REPAY => {
            let (out, repaid) =
                execute_swap_and_repay(&mut ctx, args.input_amount, args.min_output_amount)?;
            actual_output = out;
            debt_repaid = repaid;
        }
        _ => return err!(LendRescueError::ActionNotAllowed),
    }

    // 6. Recalculate post-recovery health factor on-chain from updated position state
    let post_pos = &ctx.accounts.position;
    let post_hf = Position::compute_synthetic_hf(
        post_pos.collateral_amount,
        ctx.accounts.collateral_oracle.price,
        ctx.accounts.collateral_oracle.exponent,
        ctx.accounts.market.collateral_decimals,
        post_pos.debt_amount,
        ctx.accounts.debt_oracle.price,
        ctx.accounts.debt_oracle.exponent,
        ctx.accounts.market.debt_decimals,
        ctx.accounts.market.liquidation_threshold_bps,
    )?;

    // 7. Enforce that resulting health factor satisfies the user-defined policy minimum
    require!(
        post_hf >= ctx.accounts.policy.min_resulting_hf,
        LendRescueError::ResultingHealthFactorTooLow
    );

    // 8. Consume nonce to prevent replay
    let policy = &mut ctx.accounts.policy;
    let consumed_nonce = policy.nonce;
    policy.nonce = policy
        .nonce
        .checked_add(1)
        .ok_or(LendRescueError::MathOverflow)?;

    // 9. Write immutable on-chain RecoveryReceipt
    let receipt = &mut ctx.accounts.receipt;
    receipt.position = ctx.accounts.position.key();
    receipt.owner = ctx.accounts.policy.owner;
    receipt.policy = ctx.accounts.policy.key();
    receipt.delegate = caller_key;
    receipt.action_executed = args.action;
    receipt.input_amount = actual_input;
    receipt.output_amount = actual_output;
    receipt.repayment_amount = debt_repaid;
    receipt.pre_health_factor = pre_hf;
    receipt.post_health_factor = post_hf;
    receipt.nonce_consumed = consumed_nonce;
    receipt.execution_slot = slot;
    receipt.execution_timestamp = now;
    receipt.status = 0; // Success
    receipt.version = 1;
    receipt.bump = ctx.bumps.receipt;

    // 10. Emit execution event
    emit!(RecoveryExecutedEvent {
        receipt: receipt.key(),
        position: receipt.position,
        policy: receipt.policy,
        owner: receipt.owner,
        caller: caller_key,
        action: args.action,
        input_amount: actual_input,
        output_amount: actual_output,
        pre_hf,
        post_hf,
        nonce_consumed: consumed_nonce,
        slot,
        timestamp: now,
    });

    Ok(())
}

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::constants::{MAX_ORACLE_AGE_SECS, MIN_SAFE_HF};
use crate::error::LendRescueError;
use crate::events::DemoDebtBorrowedEvent;
use crate::state::{MarketConfig, MockOracle, Position};

#[derive(Accounts)]
pub struct BorrowDemoDebt<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = owner @ LendRescueError::InvalidPositionOwner,
        has_one = market
    )]
    pub position: Account<'info, Position>,

    pub market: Account<'info, MarketConfig>,

    #[account(
        mut,
        constraint = debt_mint.key() == market.debt_mint @ LendRescueError::InvalidMint
    )]
    pub debt_mint: Account<'info, Mint>,

    #[account(
        constraint = collateral_oracle.key() == market.collateral_oracle @ LendRescueError::OracleAssetMismatch
    )]
    pub collateral_oracle: Account<'info, MockOracle>,

    #[account(
        constraint = debt_oracle.key() == market.debt_oracle @ LendRescueError::OracleAssetMismatch
    )]
    pub debt_oracle: Account<'info, MockOracle>,

    #[account(
        mut,
        constraint = owner_debt_account.owner == owner.key() @ LendRescueError::UnauthorizedCaller,
        constraint = owner_debt_account.mint == market.debt_mint @ LendRescueError::InvalidMint
    )]
    pub owner_debt_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_borrow_demo_debt(ctx: Context<BorrowDemoDebt>, amount: u64) -> Result<()> {
    require!(
        !ctx.accounts.market.is_paused,
        LendRescueError::MarketPaused
    );
    require!(amount > 0, LendRescueError::ZeroAmount);

    let now = Clock::get()?.unix_timestamp;
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

    let position = &ctx.accounts.position;
    require!(
        position.collateral_amount > 0,
        LendRescueError::InsufficientCollateral
    );

    let proposed_debt = position
        .debt_amount
        .checked_add(amount)
        .ok_or(LendRescueError::MathOverflow)?;

    let market = &ctx.accounts.market;
    let proposed_hf = Position::compute_synthetic_hf(
        position.collateral_amount,
        ctx.accounts.collateral_oracle.price,
        ctx.accounts.collateral_oracle.exponent,
        market.collateral_decimals,
        proposed_debt,
        ctx.accounts.debt_oracle.price,
        ctx.accounts.debt_oracle.exponent,
        market.debt_decimals,
        market.liquidation_threshold_bps,
    )?;

    require!(
        proposed_hf >= MIN_SAFE_HF,
        LendRescueError::BorrowLimitExceeded
    );

    // Mint synthetic debt tokens to the borrower using market PDA authority
    let market_id_bytes = market.market_id.to_le_bytes();
    let seeds = &[
        crate::constants::MARKET_SEED,
        market_id_bytes.as_ref(),
        &[market.bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = MintTo {
        mint: ctx.accounts.debt_mint.to_account_info(),
        to: ctx.accounts.owner_debt_account.to_account_info(),
        authority: ctx.accounts.market.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi_accounts, signer);
    token::mint_to(cpi_ctx, amount)?;

    let position = &mut ctx.accounts.position;
    position.debt_amount = proposed_debt;

    emit!(DemoDebtBorrowedEvent {
        position: position.key(),
        owner: position.owner,
        amount,
        total_debt: position.debt_amount,
        timestamp: now,
    });

    Ok(())
}

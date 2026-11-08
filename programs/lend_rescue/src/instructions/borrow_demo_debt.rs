use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::constants::{MARKET_SEED, MAX_ORACLE_AGE_SECS, MIN_SAFE_HF};
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

impl<'info> BorrowDemoDebt<'info> {
    pub fn borrow_demo_debt(&mut self, amount: u64) -> Result<()> {
        require!(!self.market.is_paused, LendRescueError::MarketPaused);
        require!(amount > 0, LendRescueError::ZeroAmount);

        let now = Clock::get()?.unix_timestamp;
        require!(
            self.collateral_oracle.is_fresh(now, MAX_ORACLE_AGE_SECS),
            LendRescueError::StaleOracle
        );
        require!(
            self.debt_oracle.is_fresh(now, MAX_ORACLE_AGE_SECS),
            LendRescueError::StaleOracle
        );

        let position = &self.position;
        let new_debt = position
            .debt_amount
            .checked_add(amount)
            .ok_or(LendRescueError::MathOverflow)?;

        let market = &self.market;
        let resulting_hf = Position::compute_synthetic_hf(
            position.collateral_amount,
            self.collateral_oracle.price,
            self.collateral_oracle.exponent,
            market.collateral_decimals,
            new_debt,
            self.debt_oracle.price,
            self.debt_oracle.exponent,
            market.debt_decimals,
            market.liquidation_threshold_bps,
        )?;

        require!(
            resulting_hf >= MIN_SAFE_HF,
            LendRescueError::BorrowLimitExceeded
        );

        // CPI MintTo: Mint synthetic debt tokens from market authority PDA to user debt account
        let market_id_bytes = market.market_id.to_le_bytes();
        let seeds = &[MARKET_SEED, &market_id_bytes[..], &[market.bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: self.debt_mint.to_account_info(),
            to: self.owner_debt_account.to_account_info(),
            authority: self.market.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(), cpi_accounts, signer);
        token::mint_to(cpi_ctx, amount)?;

        let position_mut = &mut self.position;
        position_mut.debt_amount = new_debt;

        emit!(DemoDebtBorrowedEvent {
            position: position_mut.key(),
            owner: position_mut.owner,
            amount,
            total_debt: new_debt,
            timestamp: now,
        });

        Ok(())
    }
}

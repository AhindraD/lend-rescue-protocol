use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::error::LendRescueError;
use crate::events::CollateralDepositedEvent;
use crate::state::{MarketConfig, Position};

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
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
        constraint = owner_collateral_account.owner == owner.key() @ LendRescueError::UnauthorizedCaller,
        constraint = owner_collateral_account.mint == market.collateral_mint @ LendRescueError::InvalidMint
    )]
    pub owner_collateral_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = collateral_vault.key() == market.collateral_vault @ LendRescueError::InvalidMint
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> DepositCollateral<'info> {
    pub fn deposit_collateral(&mut self, amount: u64) -> Result<()> {
        require!(!self.market.is_paused, LendRescueError::MarketPaused);
        require!(amount > 0, LendRescueError::ZeroAmount);

        let cpi_accounts = Transfer {
            from: self.owner_collateral_account.to_account_info(),
            to: self.collateral_vault.to_account_info(),
            authority: self.owner.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.key(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let position = &mut self.position;
        position.collateral_amount = position
            .collateral_amount
            .checked_add(amount)
            .ok_or(LendRescueError::MathOverflow)?;

        emit!(CollateralDepositedEvent {
            position: position.key(),
            owner: position.owner,
            amount,
            total_collateral: position.collateral_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

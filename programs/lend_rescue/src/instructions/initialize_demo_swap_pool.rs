use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::{DEMO_POOL_SEED, POOL_VAULT_SEED};
use crate::error::LendRescueError;
use crate::events::DemoSwapPoolInitializedEvent;
use crate::state::{DemoSwapPool, MarketConfig};

#[derive(Accounts)]
pub struct InitializeDemoSwapPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        has_one = authority @ LendRescueError::UnauthorizedMarketAuthority
    )]
    pub market: Account<'info, MarketConfig>,

    pub input_mint: Account<'info, Mint>,
    pub output_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        space = 8 + DemoSwapPool::INIT_SPACE,
        seeds = [DEMO_POOL_SEED, market.key().as_ref(), input_mint.key().as_ref(), output_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, DemoSwapPool>,

    #[account(
        init,
        payer = authority,
        token::mint = input_mint,
        token::authority = pool,
        seeds = [POOL_VAULT_SEED, pool.key().as_ref(), input_mint.key().as_ref()],
        bump
    )]
    pub input_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        token::mint = output_mint,
        token::authority = pool,
        seeds = [POOL_VAULT_SEED, pool.key().as_ref(), output_mint.key().as_ref()],
        bump
    )]
    pub output_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitializeDemoSwapPool<'info> {
    pub fn initialize_demo_swap_pool(&mut self, fee_bps: u16, bump: u8) -> Result<()> {
        require!(fee_bps <= 1_000, LendRescueError::InvalidFeeBps);
        require!(
            self.input_mint.key() != self.output_mint.key(),
            LendRescueError::InvalidMint
        );

        let pool = &mut self.pool;
        pool.market = self.market.key();
        pool.input_mint = self.input_mint.key();
        pool.output_mint = self.output_mint.key();
        pool.input_vault = self.input_vault.key();
        pool.output_vault = self.output_vault.key();
        pool.fee_bps = fee_bps;
        pool.pool_authority = pool.key();
        pool.version = 1;
        pool.bump = bump;

        emit!(DemoSwapPoolInitializedEvent {
            pool: pool.key(),
            market: pool.market,
            input_mint: pool.input_mint,
            output_mint: pool.output_mint,
            fee_bps,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

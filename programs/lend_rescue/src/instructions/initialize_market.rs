use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::{COLLATERAL_VAULT_SEED, MARKET_SEED};
use crate::error::LendRescueError;
use crate::events::MarketInitializedEvent;
use crate::state::MarketConfig;

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = MarketConfig::LEN,
        seeds = [MARKET_SEED, &market_id.to_le_bytes()],
        bump
    )]
    pub market: Account<'info, MarketConfig>,

    pub collateral_mint: Account<'info, Mint>,
    pub debt_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = collateral_mint,
        token::authority = market,
        seeds = [COLLATERAL_VAULT_SEED, market.key().as_ref()],
        bump
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle_initialize_market(
    ctx: Context<InitializeMarket>,
    market_id: u64,
    liquidation_threshold_bps: u16,
    demo_swap_fee_bps: u16,
) -> Result<()> {
    require!(
        (1_000..=9_900).contains(&liquidation_threshold_bps),
        LendRescueError::InvalidLiquidationThreshold
    );
    require!(demo_swap_fee_bps <= 1_000, LendRescueError::InvalidFeeBps);
    require!(
        ctx.accounts.collateral_mint.key() != ctx.accounts.debt_mint.key(),
        LendRescueError::InvalidMint
    );

    let market = &mut ctx.accounts.market;
    market.authority = ctx.accounts.authority.key();
    market.market_id = market_id;
    market.collateral_mint = ctx.accounts.collateral_mint.key();
    market.debt_mint = ctx.accounts.debt_mint.key();
    market.collateral_vault = ctx.accounts.collateral_vault.key();
    market.collateral_decimals = ctx.accounts.collateral_mint.decimals;
    market.debt_decimals = ctx.accounts.debt_mint.decimals;
    market.collateral_oracle = Pubkey::default();
    market.debt_oracle = Pubkey::default();
    market.liquidation_threshold_bps = liquidation_threshold_bps;
    market.demo_swap_fee_bps = demo_swap_fee_bps;
    market.is_paused = false;
    market.version = 1;
    market.bump = ctx.bumps.market;

    emit!(MarketInitializedEvent {
        market: market.key(),
        authority: ctx.accounts.authority.key(),
        collateral_mint: market.collateral_mint,
        debt_mint: market.debt_mint,
        liquidation_threshold_bps,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

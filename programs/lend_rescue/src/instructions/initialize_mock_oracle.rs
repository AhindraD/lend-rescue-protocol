use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::constants::{MAX_CONFIDENCE_BPS, ORACLE_SEED};
use crate::error::LendRescueError;
use crate::events::MockOracleInitializedEvent;
use crate::state::{MarketConfig, MockOracle};

#[derive(Accounts)]
pub struct InitializeMockOracle<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority @ LendRescueError::UnauthorizedMarketAuthority
    )]
    pub market: Account<'info, MarketConfig>,

    pub asset_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        space = MockOracle::LEN,
        seeds = [ORACLE_SEED, market.key().as_ref(), asset_mint.key().as_ref()],
        bump
    )]
    pub oracle: Account<'info, MockOracle>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_mock_oracle(
    ctx: Context<InitializeMockOracle>,
    initial_price: i64,
    exponent: i32,
    confidence: u64,
) -> Result<()> {
    require!(initial_price > 0, LendRescueError::InvalidOraclePrice);
    require!(
        (-12..=0).contains(&exponent),
        LendRescueError::InvalidDecimals
    );

    let asset_key = ctx.accounts.asset_mint.key();
    let market_key = ctx.accounts.market.key();

    let oracle = &mut ctx.accounts.oracle;
    oracle.market = market_key;
    oracle.asset_mint = asset_key;
    oracle.price = initial_price;
    oracle.exponent = exponent;
    oracle.confidence = confidence;
    oracle.last_update_slot = Clock::get()?.slot;
    oracle.last_update_timestamp = Clock::get()?.unix_timestamp;
    oracle.update_authority = ctx.accounts.authority.key();
    oracle.version = 1;
    oracle.bump = ctx.bumps.oracle;

    require!(
        oracle.is_confidence_valid(MAX_CONFIDENCE_BPS),
        LendRescueError::OracleConfidenceTooWide
    );

    let market = &mut ctx.accounts.market;
    if asset_key == market.collateral_mint {
        market.collateral_oracle = oracle.key();
    } else if asset_key == market.debt_mint {
        market.debt_oracle = oracle.key();
    }

    emit!(MockOracleInitializedEvent {
        oracle: oracle.key(),
        market: market_key,
        asset_mint: asset_key,
        price: initial_price,
        confidence,
        exponent,
        timestamp: oracle.last_update_timestamp,
    });

    Ok(())
}

use anchor_lang::prelude::*;

use crate::constants::POSITION_SEED;
use crate::error::LendRescueError;
use crate::events::PositionCreatedEvent;
use crate::state::{MarketConfig, Position};

#[derive(Accounts)]
#[instruction(position_id: u64)]
pub struct CreateDemoPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub market: Account<'info, MarketConfig>,

    #[account(
        init,
        payer = owner,
        space = Position::LEN,
        seeds = [POSITION_SEED, market.key().as_ref(), owner.key().as_ref(), &position_id.to_le_bytes()],
        bump
    )]
    pub position: Account<'info, Position>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_demo_position(
    ctx: Context<CreateDemoPosition>,
    position_id: u64,
) -> Result<()> {
    require!(
        !ctx.accounts.market.is_paused,
        LendRescueError::MarketPaused
    );

    let position = &mut ctx.accounts.position;
    position.owner = ctx.accounts.owner.key();
    position.market = ctx.accounts.market.key();
    position.position_id = position_id;
    position.collateral_amount = 0;
    position.debt_amount = 0;
    position.collateral_mint = ctx.accounts.market.collateral_mint;
    position.debt_mint = ctx.accounts.market.debt_mint;
    position.status = 0; // Active
    position.version = 1;
    position.bump = ctx.bumps.position;

    emit!(PositionCreatedEvent {
        position: position.key(),
        market: position.market,
        owner: position.owner,
        position_id,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

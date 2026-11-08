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
        space = 8 + Position::INIT_SPACE,
        seeds = [POSITION_SEED, market.key().as_ref(), owner.key().as_ref(), &position_id.to_le_bytes()],
        bump
    )]
    pub position: Account<'info, Position>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateDemoPosition<'info> {
    pub fn create_demo_position(&mut self, position_id: u64, bump: u8) -> Result<()> {
        require!(!self.market.is_paused, LendRescueError::MarketPaused);

        let position = &mut self.position;
        position.owner = self.owner.key();
        position.market = self.market.key();
        position.position_id = position_id;
        position.collateral_amount = 0;
        position.debt_amount = 0;
        position.collateral_mint = self.market.collateral_mint;
        position.debt_mint = self.market.debt_mint;
        position.status = 0; // Active
        position.version = 1;
        position.bump = bump;

        emit!(PositionCreatedEvent {
            position: position.key(),
            market: position.market,
            owner: position.owner,
            position_id,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

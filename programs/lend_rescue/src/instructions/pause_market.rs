use anchor_lang::prelude::*;

use crate::error::LendRescueError;
use crate::events::MarketPausedEvent;
use crate::state::MarketConfig;

#[derive(Accounts)]
pub struct PauseMarket<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority @ LendRescueError::UnauthorizedMarketAuthority
    )]
    pub market: Account<'info, MarketConfig>,
}

impl<'info> PauseMarket<'info> {
    pub fn pause_market(&mut self, is_paused: bool) -> Result<()> {
        let market = &mut self.market;
        market.is_paused = is_paused;

        emit!(MarketPausedEvent {
            market: market.key(),
            authority: self.authority.key(),
            is_paused,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

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

pub fn handle_pause_market(ctx: Context<PauseMarket>, is_paused: bool) -> Result<()> {
    let market = &mut ctx.accounts.market;
    market.is_paused = is_paused;

    emit!(MarketPausedEvent {
        market: market.key(),
        authority: ctx.accounts.authority.key(),
        is_paused,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

use anchor_lang::prelude::*;

use crate::constants::MAX_CONFIDENCE_BPS;
use crate::error::LendRescueError;
use crate::events::MockOracleUpdatedEvent;
use crate::state::MockOracle;

#[derive(Accounts)]
pub struct UpdateMockPrice<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = update_authority @ LendRescueError::UnauthorizedOracleUpdater
    )]
    pub oracle: Account<'info, MockOracle>,
    pub update_authority: SystemAccount<'info>,
}

pub fn handle_update_mock_price(
    ctx: Context<UpdateMockPrice>,
    new_price: i64,
    confidence: u64,
) -> Result<()> {
    require!(new_price > 0, LendRescueError::InvalidOraclePrice);

    let oracle = &mut ctx.accounts.oracle;
    let old_price = oracle.price;

    oracle.price = new_price;
    oracle.confidence = confidence;
    oracle.last_update_slot = Clock::get()?.slot;
    oracle.last_update_timestamp = Clock::get()?.unix_timestamp;

    require!(
        oracle.is_confidence_valid(MAX_CONFIDENCE_BPS),
        LendRescueError::OracleConfidenceTooWide
    );

    emit!(MockOracleUpdatedEvent {
        oracle: oracle.key(),
        market: oracle.market,
        asset_mint: oracle.asset_mint,
        old_price,
        new_price,
        confidence,
        timestamp: oracle.last_update_timestamp,
    });

    Ok(())
}

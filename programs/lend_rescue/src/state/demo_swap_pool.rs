use crate::constants::BPS_DIVISOR;
use crate::error::LendRescueError;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct DemoSwapPool {
    /// Associated market
    pub market: Pubkey,
    /// Token mint accepted as input (e.g. sSOL)
    pub input_mint: Pubkey,
    /// Token mint delivered as output (e.g. sUSDC)
    pub output_mint: Pubkey,
    /// Vault holding input token reserves
    pub input_vault: Pubkey,
    /// Vault holding output token reserves
    pub output_vault: Pubkey,
    /// Fee in basis points (e.g. 30 = 0.30%)
    pub fee_bps: u16,
    /// PDA pool authority for signing vault withdrawals
    pub pool_authority: Pubkey,
    /// Schema version
    pub version: u8,
    /// PDA bump
    pub bump: u8,
}

impl DemoSwapPool {
    /// Calculates deterministic constant-product swap output and fee
    pub fn calculate_swap_output(
        reserve_in: u64,
        reserve_out: u64,
        amount_in: u64,
        fee_bps: u16,
    ) -> Result<(u64, u64)> {
        require!(amount_in > 0, LendRescueError::ZeroAmount);
        require!(
            reserve_in > 0 && reserve_out > 0,
            LendRescueError::InsufficientPoolReserves
        );

        let fee_multiplier = (BPS_DIVISOR as u128)
            .checked_sub(fee_bps as u128)
            .ok_or(LendRescueError::MathOverflow)?;

        let fee_amount = ((amount_in as u128)
            .checked_mul(fee_bps as u128)
            .ok_or(LendRescueError::MathOverflow)?)
        .checked_div(BPS_DIVISOR as u128)
        .ok_or(LendRescueError::DivisionByZero)? as u64;

        let amount_in_with_fee = (amount_in as u128)
            .checked_mul(fee_multiplier)
            .ok_or(LendRescueError::MathOverflow)?;

        let numerator = amount_in_with_fee
            .checked_mul(reserve_out as u128)
            .ok_or(LendRescueError::MathOverflow)?;

        let denominator = (reserve_in as u128)
            .checked_mul(BPS_DIVISOR as u128)
            .ok_or(LendRescueError::MathOverflow)?
            .checked_add(amount_in_with_fee)
            .ok_or(LendRescueError::MathOverflow)?;

        require!(denominator > 0, LendRescueError::DivisionByZero);
        let amount_out_u128 = numerator
            .checked_div(denominator)
            .ok_or(LendRescueError::DivisionByZero)?;
        let amount_out = amount_out_u128 as u64;

        require!(
            amount_out > 0 && amount_out < reserve_out,
            LendRescueError::InsufficientPoolReserves
        );

        Ok((amount_out, fee_amount))
    }
}

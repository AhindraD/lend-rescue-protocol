use crate::error::LendRescueError;
use anchor_lang::prelude::*;

#[account]
pub struct Position {
    /// Owner of the lending position
    pub owner: Pubkey,
    /// Associated market
    pub market: Pubkey,
    /// Identifier for this position
    pub position_id: u64,
    /// Deposited collateral balance in base units
    pub collateral_amount: u64,
    /// Outstanding synthetic debt balance in base units
    pub debt_amount: u64,
    /// Token mint for collateral
    pub collateral_mint: Pubkey,
    /// Token mint for debt
    pub debt_mint: Pubkey,
    /// Position status (0 = Active, 1 = Closed)
    pub status: u8,
    /// Schema version
    pub version: u8,
    /// PDA bump
    pub bump: u8,
}

impl Position {
    pub const LEN: usize = 8 // discriminator
        + 32 // owner
        + 32 // market
        + 8  // position_id
        + 8  // collateral_amount
        + 8  // debt_amount
        + 32 // collateral_mint
        + 32 // debt_mint
        + 1  // status
        + 1  // version
        + 1  // bump
        + 32; // reserved padding

    /// Computes synthetic health factor scaled by HF_PRECISION (10_000 = 1.0000)
    /// Formula: (collateral_value * liquidation_threshold_bps) / debt_value
    #[allow(clippy::too_many_arguments)]
    pub fn compute_synthetic_hf(
        collateral_amount: u64,
        collateral_price: i64,
        collateral_exp: i32,
        collateral_decimals: u8,
        debt_amount: u64,
        debt_price: i64,
        debt_exp: i32,
        debt_decimals: u8,
        liquidation_threshold_bps: u16,
    ) -> Result<u64> {
        if debt_amount == 0 {
            // No debt liabilities -> safely max health factor
            return Ok(u64::MAX);
        }
        if collateral_amount == 0 || collateral_price <= 0 {
            return Ok(0);
        }
        require!(debt_price > 0, LendRescueError::InvalidOraclePrice);

        // Effective exponent power for collateral: S_c = collateral_exp - collateral_decimals
        // Effective exponent power for debt: S_d = debt_exp - debt_decimals
        let s_c = (collateral_exp as i64)
            .checked_sub(collateral_decimals as i64)
            .ok_or(LendRescueError::MathOverflow)?;
        let s_d = (debt_exp as i64)
            .checked_sub(debt_decimals as i64)
            .ok_or(LendRescueError::MathOverflow)?;

        let mut num = (collateral_amount as u128)
            .checked_mul(collateral_price as u128)
            .ok_or(LendRescueError::MathOverflow)?
            .checked_mul(liquidation_threshold_bps as u128)
            .ok_or(LendRescueError::MathOverflow)?;

        let mut den = (debt_amount as u128)
            .checked_mul(debt_price as u128)
            .ok_or(LendRescueError::MathOverflow)?;

        if s_c > s_d {
            let diff = (s_c - s_d) as u32;
            let factor = 10u128
                .checked_pow(diff)
                .ok_or(LendRescueError::MathOverflow)?;
            num = num
                .checked_mul(factor)
                .ok_or(LendRescueError::MathOverflow)?;
        } else if s_d > s_c {
            let diff = (s_d - s_c) as u32;
            let factor = 10u128
                .checked_pow(diff)
                .ok_or(LendRescueError::MathOverflow)?;
            den = den
                .checked_mul(factor)
                .ok_or(LendRescueError::MathOverflow)?;
        }

        require!(den > 0, LendRescueError::DivisionByZero);
        let hf_u128 = num
            .checked_div(den)
            .ok_or(LendRescueError::DivisionByZero)?;
        if hf_u128 > (u64::MAX as u128) {
            Ok(u64::MAX)
        } else {
            Ok(hf_u128 as u64)
        }
    }
}

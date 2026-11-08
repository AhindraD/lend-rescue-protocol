use domain_types::DomainError;
use risk_engine::fixed_point::BPS_DIVISOR;

/// Swap pool reserves and fee parameters for recovery calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwapPoolParams {
    pub reserve_in: u64,
    pub reserve_out: u64,
    pub fee_bps: u16,
}

impl SwapPoolParams {
    pub fn new(reserve_in: u64, reserve_out: u64, fee_bps: u16) -> Self {
        Self {
            reserve_in,
            reserve_out,
            fee_bps,
        }
    }
}

/// Computes swap output and fee for a constant-product pool matching on-chain `DemoSwapPool`.
pub fn calculate_swap_output(
    params: &SwapPoolParams,
    amount_in: u64,
) -> Result<(u64, u64), DomainError> {
    if amount_in == 0 {
        return Ok((0, 0));
    }
    if params.reserve_in == 0 || params.reserve_out == 0 {
        return Err(DomainError::MathOverflow);
    }
    if params.fee_bps >= BPS_DIVISOR as u16 {
        return Err(DomainError::InvalidPolicy(
            "Swap fee bps cannot exceed 10,000",
        ));
    }

    let fee_multiplier = (BPS_DIVISOR as u128)
        .checked_sub(params.fee_bps as u128)
        .ok_or(DomainError::MathOverflow)?;

    let fee_amount = ((amount_in as u128)
        .checked_mul(params.fee_bps as u128)
        .ok_or(DomainError::MathOverflow)?)
    .checked_div(BPS_DIVISOR as u128)
    .ok_or(DomainError::DivisionByZero)? as u64;

    let amount_in_with_fee = (amount_in as u128)
        .checked_mul(fee_multiplier)
        .ok_or(DomainError::MathOverflow)?;

    let numerator = amount_in_with_fee
        .checked_mul(params.reserve_out as u128)
        .ok_or(DomainError::MathOverflow)?;

    let denominator = (params.reserve_in as u128)
        .checked_mul(BPS_DIVISOR as u128)
        .ok_or(DomainError::MathOverflow)?
        .checked_add(amount_in_with_fee)
        .ok_or(DomainError::MathOverflow)?;

    if denominator == 0 {
        return Err(DomainError::DivisionByZero);
    }

    let amount_out_u128 = numerator / denominator;
    let amount_out = amount_out_u128 as u64;

    if amount_out >= params.reserve_out {
        return Err(DomainError::MathOverflow);
    }

    Ok((amount_out, fee_amount))
}

/// Calculates estimated price impact / slippage in basis points relative to spot price.
pub fn calculate_slippage_bps(
    params: &SwapPoolParams,
    amount_in: u64,
    amount_out: u64,
) -> Result<u16, DomainError> {
    if amount_in == 0 || amount_out == 0 {
        return Ok(0);
    }
    // Spot execution without price impact: ideal_out = (amount_in * reserve_out) / reserve_in
    let ideal_out = (amount_in as u128)
        .checked_mul(params.reserve_out as u128)
        .ok_or(DomainError::MathOverflow)?
        .checked_div(params.reserve_in as u128)
        .ok_or(DomainError::DivisionByZero)?;

    if ideal_out == 0 {
        return Ok(0);
    }

    let actual_out = amount_out as u128;
    if actual_out >= ideal_out {
        return Ok(0);
    }

    let diff = ideal_out - actual_out;
    let slippage_bps = diff
        .checked_mul(BPS_DIVISOR as u128)
        .ok_or(DomainError::MathOverflow)?
        .checked_div(ideal_out)
        .ok_or(DomainError::DivisionByZero)?;

    if slippage_bps > (u16::MAX as u128) {
        Ok(u16::MAX)
    } else {
        Ok(slippage_bps as u16)
    }
}

/// Computes minimum output amount after deducting maximum permitted slippage bps.
pub fn compute_min_output(amount_out: u64, max_slippage_bps: u16) -> Result<u64, DomainError> {
    if max_slippage_bps >= BPS_DIVISOR as u16 {
        return Ok(0);
    }
    let multiplier = (BPS_DIVISOR as u128) - (max_slippage_bps as u128);
    let min_out = (amount_out as u128)
        .checked_mul(multiplier)
        .ok_or(DomainError::MathOverflow)?
        .checked_div(BPS_DIVISOR as u128)
        .ok_or(DomainError::DivisionByZero)?;

    Ok(min_out as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_product_swap() {
        // Pool: 100 SOL (9 decimals) and 15,000 USDC (6 decimals), 30 bps fee (0.3%)
        let params = SwapPoolParams::new(100_000_000_000, 15_000_000_000, 30);
        // Swap 1 SOL = 1_000_000_000 raw units
        let (out, fee) = calculate_swap_output(&params, 1_000_000_000).unwrap();
        assert!(out > 0);
        assert!(fee > 0);
        // Roughly ~148 USDC output (148_000_000)
        assert!(out > 140_000_000 && out < 150_000_000);

        let slippage = calculate_slippage_bps(&params, 1_000_000_000, out).unwrap();
        assert!(slippage > 0 && slippage < 200); // Expect modest slippage (~1-2%)

        let min_out = compute_min_output(out, 100).unwrap(); // 1% slippage
        assert!(min_out < out);
    }
}

use domain_types::{DomainError, Price, TokenAmount};

use crate::decimal_math::{calculate_quote_value, DEFAULT_QUOTE_DECIMALS};
use crate::fixed_point::{mul_bps, BPS_DIVISOR};

/// Calculates net equity in quote currency (Collateral Value - Debt Value).
/// If debt exceeds collateral, returns negative equity as a signed i128.
pub fn compute_net_equity(
    collateral: &TokenAmount,
    collateral_price: &Price,
    debt: &TokenAmount,
    debt_price: &Price,
    quote_decimals: Option<u8>,
) -> Result<i128, DomainError> {
    let q_dec = quote_decimals.unwrap_or(DEFAULT_QUOTE_DECIMALS);
    let col_val = calculate_quote_value(collateral, collateral_price, q_dec)? as i128;
    let debt_val = calculate_quote_value(debt, debt_price, q_dec)? as i128;
    col_val
        .checked_sub(debt_val)
        .ok_or(DomainError::MathOverflow)
}

/// Calculates current Loan-To-Value (LTV) in basis points: (Debt Value / Collateral Value) * 10,000.
pub fn compute_ltv_bps(
    collateral: &TokenAmount,
    collateral_price: &Price,
    debt: &TokenAmount,
    debt_price: &Price,
) -> Result<u16, DomainError> {
    let col_val = calculate_quote_value(collateral, collateral_price, DEFAULT_QUOTE_DECIMALS)?;
    if col_val == 0 {
        if debt.raw == 0 {
            return Ok(0);
        }
        return Ok(u16::MAX);
    }
    let debt_val = calculate_quote_value(debt, debt_price, DEFAULT_QUOTE_DECIMALS)?;
    let ltv = debt_val
        .checked_mul(BPS_DIVISOR as u128)
        .ok_or(DomainError::MathOverflow)?
        .checked_div(col_val)
        .ok_or(DomainError::DivisionByZero)?;

    if ltv > (u16::MAX as u128) {
        Ok(u16::MAX)
    } else {
        Ok(ltv as u16)
    }
}

/// Calculates remaining borrow capacity in quote currency under the given liquidation threshold:
/// Borrow Capacity = max(0, (Collateral Value * LT) / 10,000 - Debt Value)
pub fn compute_borrow_capacity(
    collateral: &TokenAmount,
    collateral_price: &Price,
    debt: &TokenAmount,
    debt_price: &Price,
    liquidation_threshold_bps: u16,
    quote_decimals: Option<u8>,
) -> Result<u128, DomainError> {
    let q_dec = quote_decimals.unwrap_or(DEFAULT_QUOTE_DECIMALS);
    let col_val = calculate_quote_value(collateral, collateral_price, q_dec)?;
    let max_borrow = mul_bps(col_val, liquidation_threshold_bps)?;
    let debt_val = calculate_quote_value(debt, debt_price, q_dec)?;
    Ok(max_borrow.saturating_sub(debt_val))
}

/// Calculates percentage distance to liquidation in basis points.
/// Positive means safety margin above liquidation price (e.g. +588 bps = 5.88% price drop until liquidation).
/// Negative means position is already below liquidation price.
pub fn compute_distance_to_liquidation_bps(
    current_price: &Price,
    liquidation_price: &Price,
) -> Result<i64, DomainError> {
    if current_price.mantissa <= 0 {
        return Err(DomainError::InvalidPrice(
            "Current price must be strictly positive",
        ));
    }

    // Align exponents if different
    let cur_scaled = current_price.to_quote_scale(8)? as i128;
    let liq_scaled = liquidation_price.to_quote_scale(8)? as i128;

    let diff = cur_scaled
        .checked_sub(liq_scaled)
        .ok_or(DomainError::MathOverflow)?;

    let bps = diff
        .checked_mul(BPS_DIVISOR as i128)
        .ok_or(DomainError::MathOverflow)?
        .checked_div(cur_scaled)
        .ok_or(DomainError::DivisionByZero)?;

    if bps > (i64::MAX as i128) || bps < (i64::MIN as i128) {
        Err(DomainError::MathOverflow)
    } else {
        Ok(bps as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::AssetId;

    #[test]
    fn test_net_equity_and_ltv() {
        // 10 sSOL @ $150 ($1,500), 1,200 sUSDC ($1,200)
        let collateral = TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol));
        let col_price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);

        let debt = TokenAmount::new(1_200_000_000, 6, Some(AssetId::SUsdc));
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        // Net equity = $1,500 - $1,200 = $300 (300_000_000 in 6 decimals)
        let equity = compute_net_equity(&collateral, &col_price, &debt, &debt_price, None).unwrap();
        assert_eq!(equity, 300_000_000);

        // LTV = 1,200 / 1,500 = 80.00% = 8000 bps
        let ltv = compute_ltv_bps(&collateral, &col_price, &debt, &debt_price).unwrap();
        assert_eq!(ltv, 8000);

        // Borrow capacity with LT = 85%: (1500 * 0.85) - 1200 = 1275 - 1200 = $75 (75_000_000)
        let capacity =
            compute_borrow_capacity(&collateral, &col_price, &debt, &debt_price, 8500, None)
                .unwrap();
        assert_eq!(capacity, 75_000_000);
    }
}

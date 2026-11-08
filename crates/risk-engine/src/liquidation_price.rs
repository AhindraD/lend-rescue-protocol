use domain_types::{DomainError, Price, TokenAmount};

use crate::fixed_point::HF_PRECISION;

/// Computes the liquidation price of the collateral asset in debt currency terms.
/// The liquidation price is the collateral price at which the position's Health Factor equals 1.0000 (10,000).
/// Formula: P_liq = (debt_amount * debt_price * HF_PRECISION * 10^(d_C - d_D + exp_D - exp_C)) / (collateral_amount * LT)
pub fn compute_liquidation_price(
    collateral: &TokenAmount,
    debt: &TokenAmount,
    debt_price: &Price,
    collateral_exponent: i32,
    liquidation_threshold_bps: u16,
) -> Result<Price, DomainError> {
    if collateral.raw == 0 {
        return Err(DomainError::InvalidAmount(
            "Collateral amount cannot be zero for liquidation price",
        ));
    }
    if liquidation_threshold_bps == 0 {
        return Err(DomainError::DivisionByZero);
    }
    if debt.raw == 0 {
        // No debt -> position cannot be liquidated at any non-negative price
        return Ok(Price::new(
            0,
            collateral_exponent,
            0,
            debt_price.timestamp,
            debt_price.slot,
        ));
    }
    if debt_price.mantissa <= 0 {
        return Err(DomainError::InvalidPrice(
            "Debt price must be strictly positive",
        ));
    }

    // Power needed: E = (d_C - d_D) + (exp_D - exp_C)
    let net_exp = (collateral.decimals as i64)
        .checked_sub(debt.decimals as i64)
        .ok_or(DomainError::MathOverflow)?
        .checked_add(debt_price.exponent as i64)
        .ok_or(DomainError::MathOverflow)?
        .checked_sub(collateral_exponent as i64)
        .ok_or(DomainError::MathOverflow)?;

    let mut num = (debt.raw as u128)
        .checked_mul(debt_price.mantissa as u128)
        .ok_or(DomainError::MathOverflow)?
        .checked_mul(HF_PRECISION as u128)
        .ok_or(DomainError::MathOverflow)?;

    if net_exp >= 0 {
        let factor = 10u128
            .checked_pow(net_exp as u32)
            .ok_or(DomainError::MathOverflow)?;
        num = num.checked_mul(factor).ok_or(DomainError::MathOverflow)?;
    } else {
        let diff = net_exp.unsigned_abs() as u32;
        let factor = 10u128.checked_pow(diff).ok_or(DomainError::MathOverflow)?;
        num /= factor;
    }

    let den = (collateral.raw as u128)
        .checked_mul(liquidation_threshold_bps as u128)
        .ok_or(DomainError::MathOverflow)?;

    if den == 0 {
        return Err(DomainError::DivisionByZero);
    }

    let mantissa_u128 = num / den;
    if mantissa_u128 > (i64::MAX as u128) {
        return Err(DomainError::MathOverflow);
    }

    Ok(Price::new(
        mantissa_u128 as i64,
        collateral_exponent,
        0,
        debt_price.timestamp,
        debt_price.slot,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::AssetId;

    #[test]
    fn test_liquidation_price_golden_scenario() {
        // Collateral: 10 sSOL (9 decimals)
        // Debt: 1,200 sUSDC (6 decimals) @ $1.00 (mantissa 1_00000000, exp -8)
        // LT = 8500 bps (85%)
        // Expected Liq Price = 1200 / (10 * 0.85) = 141.17647058...
        // Mantissa with exp -8: 14_117_647_058
        let collateral = TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol));
        let debt = TokenAmount::new(1_200_000_000, 6, Some(AssetId::SUsdc));
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        let liq_price =
            compute_liquidation_price(&collateral, &debt, &debt_price, -8, 8500).unwrap();
        assert_eq!(liq_price.mantissa, 14_117_647_058);
        assert_eq!(liq_price.exponent, -8);

        // Formatted decimal string should begin with "141.17"
        let dec_str = liq_price.to_decimal_string();
        assert!(dec_str.starts_with("141.17"));
    }

    #[test]
    fn test_liquidation_price_zero_debt() {
        let collateral = TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol));
        let debt = TokenAmount::new(0, 6, Some(AssetId::SUsdc));
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        let liq_price =
            compute_liquidation_price(&collateral, &debt, &debt_price, -8, 8500).unwrap();
        assert_eq!(liq_price.mantissa, 0);
    }
}

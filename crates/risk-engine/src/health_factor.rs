use domain_types::{DomainError, HealthFactor, Price, TokenAmount};

use crate::fixed_point::HF_PRECISION;

/// Calculates synthetic Health Factor matching on-chain Anchor contract logic.
/// Result is scaled by HF_PRECISION (10,000 = 1.0000).
/// Formula: (collateral_amount * collateral_price * liquidation_threshold_bps) / (debt_amount * debt_price)
pub fn compute_health_factor(
    collateral: &TokenAmount,
    collateral_price: &Price,
    debt: &TokenAmount,
    debt_price: &Price,
    liquidation_threshold_bps: u16,
) -> Result<HealthFactor, DomainError> {
    if debt.raw == 0 {
        return Ok(HealthFactor(u64::MAX));
    }
    if collateral.raw == 0 || collateral_price.mantissa <= 0 {
        return Ok(HealthFactor(0));
    }
    if debt_price.mantissa <= 0 {
        return Err(DomainError::InvalidPrice(
            "Debt price must be strictly positive",
        ));
    }

    // Effective exponent power for collateral: S_c = collateral_exp - collateral_decimals
    // Effective exponent power for debt: S_d = debt_exp - debt_decimals
    let s_c = (collateral_price.exponent as i64)
        .checked_sub(collateral.decimals as i64)
        .ok_or(DomainError::MathOverflow)?;
    let s_d = (debt_price.exponent as i64)
        .checked_sub(debt.decimals as i64)
        .ok_or(DomainError::MathOverflow)?;

    let mut num = (collateral.raw as u128)
        .checked_mul(collateral_price.mantissa as u128)
        .ok_or(DomainError::MathOverflow)?
        .checked_mul(liquidation_threshold_bps as u128)
        .ok_or(DomainError::MathOverflow)?;

    let mut den = (debt.raw as u128)
        .checked_mul(debt_price.mantissa as u128)
        .ok_or(DomainError::MathOverflow)?;

    if s_c > s_d {
        let diff = (s_c - s_d) as u32;
        let factor = 10u128.checked_pow(diff).ok_or(DomainError::MathOverflow)?;
        num = num.checked_mul(factor).ok_or(DomainError::MathOverflow)?;
    } else if s_d > s_c {
        let diff = (s_d - s_c) as u32;
        let factor = 10u128.checked_pow(diff).ok_or(DomainError::MathOverflow)?;
        den = den.checked_mul(factor).ok_or(DomainError::MathOverflow)?;
    }

    if den == 0 {
        return Err(DomainError::DivisionByZero);
    }

    let hf_u128 = num / den;
    if hf_u128 > (u64::MAX as u128) {
        Ok(HealthFactor(u64::MAX))
    } else {
        Ok(HealthFactor(hf_u128 as u64))
    }
}

/// Convenience helper to check whether a position is solvent at the minimum threshold (1.0000 = 10,000).
pub fn is_position_solvent(hf: HealthFactor) -> bool {
    hf.0 >= HF_PRECISION
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::AssetId;

    #[test]
    fn test_golden_scenario_1_sol_150() {
        // Collateral: 10 sSOL (9 decimals) @ $150 (mantissa 150_00000000, exp -8)
        // Debt: 1,200 sUSDC (6 decimals) @ $1.00 (mantissa 1_00000000, exp -8)
        // Liquidation Threshold: 85% (8500 bps)
        // Collateral Value = $1,500
        // Debt Value = $1,200
        // HF = (1,500 * 0.85) / 1,200 = 1.0625 => 10,625
        let collateral = TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol));
        let col_price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);

        let debt = TokenAmount::new(1_200_000_000, 6, Some(AssetId::SUsdc));
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        let hf = compute_health_factor(&collateral, &col_price, &debt, &debt_price, 8500).unwrap();
        assert_eq!(hf.raw(), 10_625);
        assert_eq!(hf.to_decimal_string(), "1.0625");
        assert!(hf.is_solvent());
        assert!(!hf.is_liquidatable());
    }

    #[test]
    fn test_golden_scenario_2_sol_120_liquidatable() {
        // Collateral: 10 sSOL (9 decimals) @ $120 (mantissa 120_00000000, exp -8)
        // Debt: 1,200 sUSDC (6 decimals) @ $1.00 (mantissa 1_00000000, exp -8)
        // Liquidation Threshold: 85% (8500 bps)
        // Collateral Value = $1,200
        // Debt Value = $1,200
        // HF = (1,200 * 0.85) / 1,200 = 0.8500 => 8,500
        let collateral = TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol));
        let col_price = Price::new(120_00000000, -8, 1_00000000, 1700000000, 100);

        let debt = TokenAmount::new(1_200_000_000, 6, Some(AssetId::SUsdc));
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        let hf = compute_health_factor(&collateral, &col_price, &debt, &debt_price, 8500).unwrap();
        assert_eq!(hf.raw(), 8_500);
        assert_eq!(hf.to_decimal_string(), "0.8500");
        assert!(!hf.is_solvent());
        assert!(hf.is_liquidatable());
    }

    #[test]
    fn test_zero_debt_infinite_hf() {
        let collateral = TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol));
        let col_price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);
        let debt = TokenAmount::new(0, 6, Some(AssetId::SUsdc));
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        let hf = compute_health_factor(&collateral, &col_price, &debt, &debt_price, 8500).unwrap();
        assert_eq!(hf.raw(), u64::MAX);
    }

    #[test]
    fn test_zero_collateral_zero_hf() {
        let collateral = TokenAmount::new(0, 9, Some(AssetId::SSol));
        let col_price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);
        let debt = TokenAmount::new(1_000_000_000, 6, Some(AssetId::SUsdc));
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        let hf = compute_health_factor(&collateral, &col_price, &debt, &debt_price, 8500).unwrap();
        assert_eq!(hf.raw(), 0);
    }
}

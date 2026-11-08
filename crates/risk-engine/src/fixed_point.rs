use domain_types::DomainError;

pub const BPS_DIVISOR: u64 = 10_000;
pub const HF_PRECISION: u64 = 10_000;

/// Multiplies value by bps (basis points) and divides by BPS_DIVISOR (10,000).
pub fn mul_bps(val: u128, bps: u16) -> Result<u128, DomainError> {
    val.checked_mul(bps as u128)
        .ok_or(DomainError::MathOverflow)?
        .checked_div(BPS_DIVISOR as u128)
        .ok_or(DomainError::DivisionByZero)
}

/// Computes (num * scale) / den with checked math.
pub fn mul_div(num: u128, scale: u128, den: u128) -> Result<u128, DomainError> {
    if den == 0 {
        return Err(DomainError::DivisionByZero);
    }
    num.checked_mul(scale)
        .ok_or(DomainError::MathOverflow)?
        .checked_div(den)
        .ok_or(DomainError::DivisionByZero)
}

/// Computes (num * scale + den - 1) / den (ceiling division) with checked math.
pub fn mul_div_ceil(num: u128, scale: u128, den: u128) -> Result<u128, DomainError> {
    if den == 0 {
        return Err(DomainError::DivisionByZero);
    }
    let prod = num.checked_mul(scale).ok_or(DomainError::MathOverflow)?;
    let num_rounded = prod
        .checked_add(den.saturating_sub(1))
        .ok_or(DomainError::MathOverflow)?;
    Ok(num_rounded / den)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_bps() {
        assert_eq!(mul_bps(100_0000, 8500).unwrap(), 85_0000);
        assert_eq!(mul_bps(1000, 10000).unwrap(), 1000);
        assert_eq!(mul_bps(1000, 0).unwrap(), 0);
    }

    #[test]
    fn test_mul_div() {
        assert_eq!(mul_div(100, 200, 50).unwrap(), 400);
        assert!(mul_div(100, 200, 0).is_err());
    }

    #[test]
    fn test_mul_div_ceil() {
        assert_eq!(mul_div_ceil(10, 1, 3).unwrap(), 4);
        assert_eq!(mul_div(10, 1, 3).unwrap(), 3);
    }
}

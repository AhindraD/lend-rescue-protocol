use domain_types::{DomainError, Price};

pub const DEFAULT_MAX_ORACLE_AGE_SECS: i64 = 60;
pub const DEFAULT_MAX_CONFIDENCE_BPS: u64 = 200; // 2%
pub const MAX_ALLOWED_CLOCK_SKEW_SECS: i64 = 5;

/// Validates an oracle price against freshness, confidence, exponent, and value sanity bounds.
pub fn validate_oracle_price(
    price: &Price,
    current_timestamp: i64,
    max_age_secs: Option<i64>,
    max_confidence_bps: Option<u64>,
) -> Result<(), DomainError> {
    if price.mantissa <= 0 {
        return Err(DomainError::OracleValidationFailed(
            "Oracle price must be strictly positive",
        ));
    }

    if price.exponent < -12 || price.exponent > 0 {
        return Err(DomainError::OracleValidationFailed(
            "Price exponent out of supported range (-12..=0)",
        ));
    }

    let max_age = max_age_secs.unwrap_or(DEFAULT_MAX_ORACLE_AGE_SECS);
    let max_conf_bps = max_confidence_bps.unwrap_or(DEFAULT_MAX_CONFIDENCE_BPS);

    // Clock skew check: oracle timestamp cannot be far in the future
    if price.timestamp > current_timestamp + MAX_ALLOWED_CLOCK_SKEW_SECS {
        return Err(DomainError::OracleValidationFailed(
            "Oracle price timestamp is in the future",
        ));
    }

    // Staleness check
    let age = current_timestamp - price.timestamp;
    if age > max_age {
        return Err(DomainError::OracleValidationFailed("Oracle price is stale"));
    }

    // Confidence interval check: confidence * 10_000 <= mantissa * max_conf_bps
    let conf_scaled = (price.confidence as u128)
        .checked_mul(10_000)
        .ok_or(DomainError::MathOverflow)?;
    let max_conf_allowed = (price.mantissa as u128)
        .checked_mul(max_conf_bps as u128)
        .ok_or(DomainError::MathOverflow)?;

    if conf_scaled > max_conf_allowed {
        return Err(DomainError::OracleValidationFailed(
            "Oracle confidence interval exceeds maximum threshold",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_oracle_price() {
        let price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);
        assert!(validate_oracle_price(&price, 1700000010, Some(60), Some(200)).is_ok());
    }

    #[test]
    fn test_stale_oracle_price() {
        let price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);
        // Age is 70s > 60s
        assert!(validate_oracle_price(&price, 1700000070, Some(60), Some(200)).is_err());
    }

    #[test]
    fn test_confidence_too_wide() {
        // Price is 100, confidence is 3 (3% > 2%)
        let price = Price::new(100_00000000, -8, 3_00000000, 1700000000, 100);
        assert!(validate_oracle_price(&price, 1700000005, Some(60), Some(200)).is_err());
    }

    #[test]
    fn test_negative_price() {
        let price = Price::new(-150_00000000, -8, 1000, 1700000000, 100);
        assert!(validate_oracle_price(&price, 1700000005, Some(60), Some(200)).is_err());
    }
}

use super::error::PythError;

/// Validates that the decoded feed ID matches the expected asset feed ID.
pub fn validate_feed_id(expected: &[u8; 32], actual: &[u8; 32]) -> Result<(), PythError> {
    if expected != actual {
        return Err(PythError::FeedIdMismatch {
            expected: hex::encode(expected),
            actual: hex::encode(actual),
        });
    }
    Ok(())
}

/// Validates that the account is owned by the expected Pyth program.
pub fn validate_account_owner(expected_program: &str, actual_owner: &str) -> Result<(), PythError> {
    if expected_program != actual_owner {
        return Err(PythError::InvalidAccountOwner {
            expected: expected_program.to_string(),
            actual: actual_owner.to_string(),
        });
    }
    Ok(())
}

/// Validates that the price mantissa is strictly positive.
pub fn validate_price_positivity(price: i64) -> Result<(), PythError> {
    if price <= 0 {
        return Err(PythError::NonPositivePrice(price));
    }
    Ok(())
}

/// Validates that the confidence interval does not exceed the allowed threshold.
/// `max_ratio_bps` = 200 means confidence cannot exceed 2% of price.
pub fn validate_confidence(
    price: i64,
    confidence: u64,
    max_ratio_bps: u64,
) -> Result<(), PythError> {
    if price <= 0 {
        return Err(PythError::NonPositivePrice(price));
    }

    let price_u128 = price as u128;
    let conf_u128 = confidence as u128;

    // confidence / price <= max_ratio_bps / 10_000
    // <=> confidence * 10_000 <= price * max_ratio_bps
    let max_permitted_conf = match price_u128.checked_mul(max_ratio_bps as u128) {
        Some(product) => product / 10_000,
        None => return Err(PythError::ConfidenceTooWide { price, confidence }),
    };

    if conf_u128 > max_permitted_conf {
        return Err(PythError::ConfidenceTooWide { price, confidence });
    }

    Ok(())
}

/// Validates that the observation is fresh and not older than max_age_secs.
pub fn validate_freshness(
    publish_time: i64,
    current_time: i64,
    max_age_secs: i64,
) -> Result<(), PythError> {
    let age_secs = current_time - publish_time;
    if age_secs > max_age_secs {
        return Err(PythError::StalePrice {
            age_secs,
            max_age_secs,
        });
    }
    Ok(())
}

/// Validates that publication time is plausible and not in the future beyond allowable clock skew.
pub fn validate_publication_time(
    publish_time: i64,
    current_time: i64,
    clock_skew_tolerance: i64,
) -> Result<(), PythError> {
    if publish_time > current_time + clock_skew_tolerance {
        return Err(PythError::FuturePublishTime {
            publish_time,
            now: current_time,
        });
    }
    Ok(())
}

/// Validates that the price exponent is supported (negative and between -14 and 0).
pub fn validate_exponent(expo: i32) -> Result<(), PythError> {
    if !(-14..=0).contains(&expo) {
        return Err(PythError::InvalidExponent(expo));
    }
    Ok(())
}

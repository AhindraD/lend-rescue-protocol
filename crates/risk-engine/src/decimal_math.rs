use domain_types::{DomainError, Price, TokenAmount};

/// Default quote precision (6 decimals: micro-USD, matching USDC base units).
pub const DEFAULT_QUOTE_DECIMALS: u8 = 6;

/// Calculates the quote currency value for a given token amount and price at the requested quote decimals.
/// Value = amount * price normalized to quote_decimals.
pub fn calculate_quote_value(
    amount: &TokenAmount,
    price: &Price,
    quote_decimals: u8,
) -> Result<u128, DomainError> {
    if amount.raw == 0 {
        return Ok(0);
    }
    if price.mantissa <= 0 {
        return Err(DomainError::InvalidPrice("Price must be strictly positive"));
    }

    let raw_amount = amount.raw as u128;
    let mantissa = price.mantissa as u128;

    let prod = raw_amount
        .checked_mul(mantissa)
        .ok_or(DomainError::MathOverflow)?;

    // Effective power: E = quote_decimals + price.exponent - amount.decimals
    let net_exp = (quote_decimals as i64)
        .checked_add(price.exponent as i64)
        .ok_or(DomainError::MathOverflow)?
        .checked_sub(amount.decimals as i64)
        .ok_or(DomainError::MathOverflow)?;

    if net_exp >= 0 {
        let factor = 10u128
            .checked_pow(net_exp as u32)
            .ok_or(DomainError::MathOverflow)?;
        prod.checked_mul(factor).ok_or(DomainError::MathOverflow)
    } else {
        let diff = net_exp.unsigned_abs() as u32;
        let factor = 10u128.checked_pow(diff).ok_or(DomainError::MathOverflow)?;
        Ok(prod / factor)
    }
}

/// Converts a quote currency amount (at quote_decimals) into token base units for a given asset and price.
/// Token base units = (quote_amount * 10^(token_decimals - price.exponent - quote_decimals)) / price.mantissa
pub fn quote_value_to_token_amount(
    quote_value: u128,
    token_decimals: u8,
    price: &Price,
    quote_decimals: u8,
) -> Result<u64, DomainError> {
    if quote_value == 0 {
        return Ok(0);
    }
    if price.mantissa <= 0 {
        return Err(DomainError::InvalidPrice("Price must be strictly positive"));
    }

    let mantissa = price.mantissa as u128;

    // Power needed: E = token_decimals - price.exponent - quote_decimals
    let net_exp = (token_decimals as i64)
        .checked_sub(price.exponent as i64)
        .ok_or(DomainError::MathOverflow)?
        .checked_sub(quote_decimals as i64)
        .ok_or(DomainError::MathOverflow)?;

    let num = if net_exp >= 0 {
        let factor = 10u128
            .checked_pow(net_exp as u32)
            .ok_or(DomainError::MathOverflow)?;
        quote_value
            .checked_mul(factor)
            .ok_or(DomainError::MathOverflow)?
    } else {
        let diff = net_exp.unsigned_abs() as u32;
        let factor = 10u128.checked_pow(diff).ok_or(DomainError::MathOverflow)?;
        quote_value / factor
    };

    let raw = num
        .checked_div(mantissa)
        .ok_or(DomainError::DivisionByZero)?;

    if raw > (u64::MAX as u128) {
        Err(DomainError::MathOverflow)
    } else {
        Ok(raw as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::AssetId;

    #[test]
    fn test_calculate_quote_value_sol() {
        // 10 sSOL (9 decimals) at $150 (mantissa 150_00000000, exp -8)
        let amount = TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol));
        let price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);

        // Quote value in 6 decimals (micro-USD)
        let value = calculate_quote_value(&amount, &price, 6).unwrap();
        // 10 SOL * $150 = $1,500 => 1_500_000_000 in 6 decimals
        assert_eq!(value, 1_500_000_000);
    }

    #[test]
    fn test_calculate_quote_value_usdc() {
        // 1,200 sUSDC (6 decimals) at $1.00 (mantissa 1_00000000, exp -8)
        let amount = TokenAmount::new(1_200_000_000, 6, Some(AssetId::SUsdc));
        let price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        let value = calculate_quote_value(&amount, &price, 6).unwrap();
        // 1,200 USDC * $1.00 = $1,200 => 1_200_000_000 in 6 decimals
        assert_eq!(value, 1_200_000_000);
    }

    #[test]
    fn test_quote_value_to_token_amount() {
        let price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);
        // $1,500 quote value (6 decimals) at $150/SOL should equal 10 SOL (9 decimals)
        let raw_sol = quote_value_to_token_amount(1_500_000_000, 9, &price, 6).unwrap();
        assert_eq!(raw_sol, 10_000_000_000);
    }
}

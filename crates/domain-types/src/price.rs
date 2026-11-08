use core::fmt;
use serde::{Deserialize, Serialize};

use crate::asset::AssetId;
use crate::error::DomainError;

/// Oracle price representation carrying mantissa, exponent, confidence interval, and timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Price {
    /// Asset this price applies to
    pub asset_id: Option<AssetId>,
    /// Raw price mantissa (e.g. 150_00000000 for $150 with exponent -8)
    pub mantissa: i64,
    /// Exponent scaling factor (typically negative, e.g. -8 for 8 decimals)
    pub exponent: i32,
    /// Absolute confidence interval
    pub confidence: u64,
    /// Publication Unix timestamp (seconds)
    pub timestamp: i64,
    /// Publication slot
    pub slot: u64,
}

impl Price {
    pub const fn new(
        mantissa: i64,
        exponent: i32,
        confidence: u64,
        timestamp: i64,
        slot: u64,
    ) -> Self {
        Self {
            asset_id: None,
            mantissa,
            exponent,
            confidence,
            timestamp,
            slot,
        }
    }

    pub fn with_asset(mut self, asset_id: AssetId) -> Self {
        self.asset_id = Some(asset_id);
        self
    }

    /// Checks if the price mantissa is strictly positive.
    pub const fn is_positive(&self) -> bool {
        self.mantissa > 0
    }

    /// Normalizes the price to a target integer scale (e.g. target_decimals = 8 => 10^8).
    pub fn to_quote_scale(&self, target_decimals: u8) -> Result<u128, DomainError> {
        if self.mantissa <= 0 {
            return Err(DomainError::InvalidPrice("Price must be strictly positive"));
        }
        let mantissa_u128 = self.mantissa as u128;
        let price_decimals = (-self.exponent) as u8;

        if target_decimals >= price_decimals {
            let diff = (target_decimals - price_decimals) as u32;
            let factor = 10u128.checked_pow(diff).ok_or(DomainError::MathOverflow)?;
            mantissa_u128
                .checked_mul(factor)
                .ok_or(DomainError::MathOverflow)
        } else {
            let diff = (price_decimals - target_decimals) as u32;
            let factor = 10u128.checked_pow(diff).ok_or(DomainError::MathOverflow)?;
            Ok(mantissa_u128 / factor)
        }
    }

    /// Converts price to formatted human decimal string (e.g. "150.00000000").
    pub fn to_decimal_string(&self) -> String {
        if self.exponent >= 0 {
            let factor = 10u64.pow(self.exponent as u32);
            format!("{}", (self.mantissa as u64) * factor)
        } else {
            let dec = (-self.exponent) as usize;
            let factor = 10u64.pow(dec as u32);
            let raw = self.mantissa.unsigned_abs();
            let whole = raw / factor;
            let frac = raw % factor;
            format!("{:01}.{:0width$}", whole, frac, width = dec)
        }
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref asset) = self.asset_id {
            write!(f, "${} ({})", self.to_decimal_string(), asset)
        } else {
            write!(f, "${}", self.to_decimal_string())
        }
    }
}

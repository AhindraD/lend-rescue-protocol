use core::fmt;
use serde::{Deserialize, Serialize};

use crate::asset::AssetId;
use crate::error::DomainError;

/// Strongly-typed token amount representing raw base units with associated decimals.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TokenAmount {
    /// Raw integer amount in token base units (e.g. lamports, micro-USDC)
    pub raw: u64,
    /// Number of decimals for the asset (e.g. 9 for SOL, 6 for USDC)
    pub decimals: u8,
    /// Optional asset identifier
    pub asset_id: Option<AssetId>,
}

impl TokenAmount {
    /// Creates a new token amount with specified raw units and decimals.
    pub const fn new(raw: u64, decimals: u8, asset_id: Option<AssetId>) -> Self {
        Self {
            raw,
            decimals,
            asset_id,
        }
    }

    /// Creates a token amount with raw units and decimals, no asset ID.
    pub const fn from_raw(raw: u64, decimals: u8) -> Self {
        Self {
            raw,
            decimals,
            asset_id: None,
        }
    }

    /// Creates a zero amount for the specified decimals.
    pub const fn zero(decimals: u8) -> Self {
        Self {
            raw: 0,
            decimals,
            asset_id: None,
        }
    }

    /// Checks if the amount is zero.
    pub const fn is_zero(&self) -> bool {
        self.raw == 0
    }

    /// Checked addition of two token amounts with identical decimals.
    pub fn checked_add(&self, other: &Self) -> Result<Self, DomainError> {
        if self.decimals != other.decimals {
            return Err(DomainError::DecimalMismatch {
                expected: self.decimals,
                actual: other.decimals,
            });
        }
        let raw = self
            .raw
            .checked_add(other.raw)
            .ok_or(DomainError::MathOverflow)?;
        Ok(Self {
            raw,
            decimals: self.decimals,
            asset_id: self.asset_id.clone().or_else(|| other.asset_id.clone()),
        })
    }

    /// Checked subtraction of two token amounts with identical decimals.
    pub fn checked_sub(&self, other: &Self) -> Result<Self, DomainError> {
        if self.decimals != other.decimals {
            return Err(DomainError::DecimalMismatch {
                expected: self.decimals,
                actual: other.decimals,
            });
        }
        let raw = self
            .raw
            .checked_sub(other.raw)
            .ok_or(DomainError::MathOverflow)?;
        Ok(Self {
            raw,
            decimals: self.decimals,
            asset_id: self.asset_id.clone().or_else(|| other.asset_id.clone()),
        })
    }

    /// Checked scalar multiplication.
    pub fn checked_mul_scalar(&self, scalar: u64) -> Result<Self, DomainError> {
        let raw = self
            .raw
            .checked_mul(scalar)
            .ok_or(DomainError::MathOverflow)?;
        Ok(Self {
            raw,
            decimals: self.decimals,
            asset_id: self.asset_id.clone(),
        })
    }

    /// Checked scalar division.
    pub fn checked_div_scalar(&self, divisor: u64) -> Result<Self, DomainError> {
        if divisor == 0 {
            return Err(DomainError::DivisionByZero);
        }
        Ok(Self {
            raw: self.raw / divisor,
            decimals: self.decimals,
            asset_id: self.asset_id.clone(),
        })
    }

    /// Normalizes the token amount to a target decimal precision without floating point.
    pub fn to_target_decimals(&self, target_decimals: u8) -> Result<u128, DomainError> {
        let raw_u128 = self.raw as u128;
        if target_decimals >= self.decimals {
            let diff = (target_decimals - self.decimals) as u32;
            let factor = 10u128.checked_pow(diff).ok_or(DomainError::MathOverflow)?;
            raw_u128
                .checked_mul(factor)
                .ok_or(DomainError::MathOverflow)
        } else {
            let diff = (self.decimals - target_decimals) as u32;
            let factor = 10u128.checked_pow(diff).ok_or(DomainError::MathOverflow)?;
            Ok(raw_u128 / factor)
        }
    }

    /// Returns human-readable string representation without floating-point loss.
    /// Example: 10_000_000_000 raw with 9 decimals => "10.000000000"
    pub fn to_decimal_string(&self) -> String {
        if self.decimals == 0 {
            return self.raw.to_string();
        }
        let factor = 10u64.pow(self.decimals as u32);
        let whole = self.raw / factor;
        let frac = self.raw % factor;
        format!(
            "{:01}.{:0width$}",
            whole,
            frac,
            width = self.decimals as usize
        )
    }
}

impl fmt::Display for TokenAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref asset) = self.asset_id {
            write!(f, "{} {}", self.to_decimal_string(), asset)
        } else {
            write!(
                f,
                "{} (decimals: {})",
                self.to_decimal_string(),
                self.decimals
            )
        }
    }
}

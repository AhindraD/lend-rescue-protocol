use core::fmt;
use serde::{Deserialize, Serialize};

use crate::price::Price;

/// Fixed-point health factor scaled by 10,000 (1.0000 = 10,000).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HealthFactor(pub u64);

impl HealthFactor {
    /// Fixed-point scale precision (1.0 = 10,000)
    pub const PRECISION: u64 = 10_000;

    /// Minimum threshold for solvency (1.0 = 10,000)
    pub const LIQUIDATION_THRESHOLD: u64 = 10_000;

    pub const fn from_scaled(scaled: u64) -> Self {
        Self(scaled)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }

    pub const fn is_solvent(&self) -> bool {
        self.0 >= Self::LIQUIDATION_THRESHOLD
    }

    pub const fn is_liquidatable(&self) -> bool {
        self.0 < Self::LIQUIDATION_THRESHOLD
    }

    /// Formats as human decimal string without floating point (e.g. "1.0625").
    pub fn to_decimal_string(&self) -> String {
        let whole = self.0 / Self::PRECISION;
        let frac = self.0 % Self::PRECISION;
        format!("{}.{:04}", whole, frac)
    }
}

impl fmt::Display for HealthFactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_decimal_string())
    }
}

/// Configurable risk classification thresholds (scaled by 10,000).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskThresholds {
    /// Threshold at and above which position is Healthy (default 12,500 = 1.25)
    pub healthy_min: u64,
    /// Threshold at and above which position is Warning (default 11,000 = 1.10)
    pub warning_min: u64,
    /// Threshold at and above which position is Critical (default 10,000 = 1.00)
    pub critical_min: u64,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            healthy_min: 12_500,  // 1.25
            warning_min: 11_000,  // 1.10
            critical_min: 10_000, // 1.00
        }
    }
}

/// Discrete risk bands for a lending position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskBand {
    /// HF >= 1.25: comfortable safety buffer
    Healthy,
    /// 1.10 <= HF < 1.25: elevated risk, recovery advised
    Warning,
    /// 1.00 <= HF < 1.10: immediate risk, recovery urgent
    Critical,
    /// HF < 1.00: subject to liquidation
    Liquidatable,
    /// Price or position data is stale
    Stale,
    /// Required data is unavailable
    Unavailable,
}

impl RiskBand {
    /// Classifies a health factor using given or default thresholds.
    pub fn classify(hf: HealthFactor, thresholds: Option<&RiskThresholds>) -> Self {
        let default_t = RiskThresholds::default();
        let t = thresholds.unwrap_or(&default_t);

        if hf.0 >= t.healthy_min {
            RiskBand::Healthy
        } else if hf.0 >= t.warning_min {
            RiskBand::Warning
        } else if hf.0 >= t.critical_min {
            RiskBand::Critical
        } else {
            RiskBand::Liquidatable
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            RiskBand::Healthy => "Healthy",
            RiskBand::Warning => "Warning",
            RiskBand::Critical => "Critical",
            RiskBand::Liquidatable => "Liquidatable",
            RiskBand::Stale => "Stale / Unreliable",
            RiskBand::Unavailable => "Unavailable",
        }
    }
}

impl fmt::Display for RiskBand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Comprehensive risk profile for a position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskProfile {
    pub health_factor: Option<HealthFactor>,
    pub risk_band: RiskBand,
    pub liquidation_price: Option<Price>,
    pub collateral_value_quote: u128,
    pub debt_value_quote: u128,
    pub borrow_capacity_quote: u128,
    pub distance_to_liquidation_bps: Option<i64>,
}

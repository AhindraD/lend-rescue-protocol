use serde::{Deserialize, Serialize};

use crate::amount::TokenAmount;
use crate::error::DomainError;

/// Individual hop or split step in an indicative swap route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwapRouteStep {
    pub amm_key: Option<String>,
    pub label: Option<String>,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: u64,
    pub out_amount: u64,
    pub fee_amount: u64,
    pub fee_mint: Option<String>,
    pub percent: u8,
}

/// Normalized, quote-only swap quotation for recovery planning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwapQuote {
    /// Name of quote provider (e.g. "Jupiter")
    pub provider: String,
    /// Base58 mint of input asset
    pub input_mint: String,
    /// Base58 mint of output asset
    pub output_mint: String,
    /// Input token amount with precision metadata
    pub in_amount: TokenAmount,
    /// Expected output token amount before slippage (already net of platform fee)
    pub expected_out_amount: TokenAmount,
    /// Calculated local planning floor: outAmount * (10_000 - max_slippage_bps) / 10_000
    pub planning_min_out_amount: TokenAmount,
    /// Router/pool fee in basis points
    pub fee_bps: u16,
    /// Token mint in which router fee is levied
    pub fee_mint: Option<String>,
    /// Optional integrator platform fee in basis points
    pub platform_fee_bps: Option<u16>,
    /// Optional integrator platform fee amount in base units
    pub platform_fee_amount: Option<u64>,
    /// Winning route identifier (e.g. "iris", "jupiter-v2")
    pub router: String,
    /// Execution mode (e.g. "ExactIn")
    pub mode: String,
    /// Price impact as decimal string percentage (no floating-point types)
    pub price_impact_pct: Option<String>,
    /// Detailed multi-hop route plan
    pub route_plan: Vec<SwapRouteStep>,
    /// Observation timestamp in Unix seconds
    pub observed_at: i64,
    /// Expiration timestamp in Unix seconds (observed_at + ttl)
    pub valid_until: i64,
    /// Always true: indicates quote is advisory and has not constructed an on-chain transaction
    pub is_indicative: bool,
}

impl SwapQuote {
    /// Computes the conservative local planning minimum output:
    /// `planning_min_output = outAmount * (10_000 - max_slippage_bps) / 10_000`
    ///
    /// Uses checked 128-bit integer arithmetic with floor rounding.
    pub fn compute_planning_min_output(
        out_amount: u64,
        max_slippage_bps: u16,
    ) -> Result<u64, DomainError> {
        if max_slippage_bps > 10_000 {
            return Err(DomainError::InvalidPolicy(
                "Max slippage basis points cannot exceed 10,000",
            ));
        }

        let multiplier = 10_000u128
            .checked_sub(max_slippage_bps as u128)
            .ok_or(DomainError::MathOverflow)?;

        let min_out = (out_amount as u128)
            .checked_mul(multiplier)
            .ok_or(DomainError::MathOverflow)?
            .checked_div(10_000u128)
            .ok_or(DomainError::DivisionByZero)?;

        Ok(min_out as u64)
    }

    /// Checks if the quote has expired relative to the provided Unix timestamp.
    pub fn is_stale(&self, current_timestamp: i64) -> bool {
        current_timestamp > self.valid_until
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_planning_min_output_exact_bps() {
        // 100 USDC (100_000_000 raw) with 100 bps slippage (1%)
        let min_out = SwapQuote::compute_planning_min_output(100_000_000, 100).unwrap();
        assert_eq!(min_out, 99_000_000);

        // 0 bps slippage
        let min_out_zero = SwapQuote::compute_planning_min_output(100_000_000, 0).unwrap();
        assert_eq!(min_out_zero, 100_000_000);

        // 50 bps slippage (0.5%)
        let min_out_half = SwapQuote::compute_planning_min_output(100_000_000, 50).unwrap();
        assert_eq!(min_out_half, 99_500_000);

        // 10,000 bps slippage (100%)
        let min_out_full = SwapQuote::compute_planning_min_output(100_000_000, 10_000).unwrap();
        assert_eq!(min_out_full, 0);

        // > 10,000 bps slippage must error
        assert!(SwapQuote::compute_planning_min_output(100_000_000, 10_001).is_err());
    }

    #[test]
    fn test_quote_staleness() {
        let quote = SwapQuote {
            provider: "Jupiter".to_string(),
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            in_amount: TokenAmount::new(1_000_000_000, 9, None),
            expected_out_amount: TokenAmount::new(150_000_000, 6, None),
            planning_min_out_amount: TokenAmount::new(148_500_000, 6, None),
            fee_bps: 10,
            fee_mint: None,
            platform_fee_bps: None,
            platform_fee_amount: None,
            router: "iris".to_string(),
            mode: "ExactIn".to_string(),
            price_impact_pct: Some("0.05".to_string()),
            route_plan: vec![],
            observed_at: 1000,
            valid_until: 1030,
            is_indicative: true,
        };

        assert!(!quote.is_stale(1000));
        assert!(!quote.is_stale(1030));
        assert!(quote.is_stale(1031));
    }
}


use domain_types::{AssetId, SwapQuote, SwapRouteStep, TokenAmount};
use serde::{Deserialize, Serialize};

use super::error::JupiterError;

/// Raw JSON response from Jupiter Swap API V2 `/order` endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterOrderResponseJson {
    pub in_amount: String,
    pub out_amount: String,
    #[serde(default)]
    pub fee_bps: u16,
    pub fee_mint: Option<String>,
    pub platform_fee: Option<JupiterPlatformFeeJson>,
    pub price_impact_pct: Option<String>,
    #[serde(default)]
    pub route_plan: Vec<JupiterRoutePlanStepJson>,
    pub router: Option<String>,
    pub mode: Option<String>,
    /// Must be null in quote-only mode when taker is omitted
    #[serde(default)]
    pub transaction: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterPlatformFeeJson {
    pub amount: Option<String>,
    #[serde(default)]
    pub fee_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterRoutePlanStepJson {
    pub swap_info: JupiterSwapInfoJson,
    #[serde(default)]
    pub percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterSwapInfoJson {
    pub amm_key: Option<String>,
    pub label: Option<String>,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: Option<String>,
    pub fee_mint: Option<String>,
}

/// Resolves standard token decimals and domain AssetId from mint address.
pub fn resolve_mint_asset(mint: &str) -> (u8, Option<AssetId>) {
    match mint {
        "So11111111111111111111111111111111111111112" => (9, Some(AssetId::Sol)),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => (6, Some(AssetId::Usdc)),
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => {
            (6, Some(AssetId::Custom("USDT".to_string())))
        }
        "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1" => {
            (9, Some(AssetId::Custom("bSOL".to_string())))
        }
        "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So" => {
            (9, Some(AssetId::Custom("mSOL".to_string())))
        }
        _ => (6, Some(AssetId::Custom(mint.to_string()))),
    }
}

impl JupiterOrderResponseJson {
    /// Normalizes raw Jupiter response into LendRescue domain `SwapQuote`.
    ///
    /// Validates quote-only security invariants:
    /// 1. Verifies that `transaction` is null or None (no transaction built).
    /// 2. Verifies that `inAmount` matches the requested amount.
    /// 3. Normalizes amounts into pure integer domain types without floating-point math.
    /// 4. Computes conservative local planning minimum output floor.
    pub fn into_swap_quote(
        self,
        input_mint: &str,
        output_mint: &str,
        expected_raw_in: u64,
        max_slippage_bps: u16,
        observed_at: i64,
        ttl_secs: i64,
    ) -> Result<SwapQuote, JupiterError> {
        // Security assertion: Reject if an unexpected transaction payload is returned
        if let Some(ref tx) = self.transaction {
            if !tx.is_null() {
                return Err(JupiterError::UnexpectedTransactionPayload);
            }
        }

        // Validate raw input amount
        let in_amount_raw = self
            .in_amount
            .parse::<u64>()
            .map_err(|e| JupiterError::InvalidAmount(format!("Invalid inAmount: {}", e)))?;

        if in_amount_raw != expected_raw_in {
            return Err(JupiterError::InvalidAmount(format!(
                "inAmount mismatch: requested {}, received {}",
                expected_raw_in, in_amount_raw
            )));
        }

        // Validate raw output amount
        let out_amount_raw = self
            .out_amount
            .parse::<u64>()
            .map_err(|e| JupiterError::InvalidAmount(format!("Invalid outAmount: {}", e)))?;

        if out_amount_raw == 0 {
            return Err(JupiterError::NoRouteFound(
                "Jupiter returned zero output amount for swap".to_string(),
            ));
        }

        // Resolve decimals and token amounts
        let (in_decimals, in_asset) = resolve_mint_asset(input_mint);
        let (out_decimals, out_asset) = resolve_mint_asset(output_mint);

        let in_amount = TokenAmount::new(in_amount_raw, in_decimals, in_asset);
        let expected_out_amount = TokenAmount::new(out_amount_raw, out_decimals, out_asset.clone());

        // Compute conservative planning minimum output floor using integer math:
        // planning_min = outAmount * (10_000 - max_slippage_bps) / 10_000
        let planning_min_raw =
            SwapQuote::compute_planning_min_output(out_amount_raw, max_slippage_bps)
                .map_err(|e| JupiterError::InvalidAmount(e.to_string()))?;
        let planning_min_out_amount = TokenAmount::new(planning_min_raw, out_decimals, out_asset);

        // Parse platform fee
        let (platform_fee_bps, platform_fee_amount) = match self.platform_fee {
            Some(pf) => {
                let amount_parsed = pf.amount.and_then(|s| s.parse::<u64>().ok());
                (Some(pf.fee_bps), amount_parsed)
            }
            None => (None, None),
        };

        // Normalize route plan steps
        let route_plan = self
            .route_plan
            .into_iter()
            .map(|step| {
                let in_amt = step.swap_info.in_amount.parse::<u64>().unwrap_or(0);
                let out_amt = step.swap_info.out_amount.parse::<u64>().unwrap_or(0);
                let fee_amt = step
                    .swap_info
                    .fee_amount
                    .as_deref()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);

                SwapRouteStep {
                    amm_key: step.swap_info.amm_key,
                    label: step.swap_info.label,
                    input_mint: step.swap_info.input_mint,
                    output_mint: step.swap_info.output_mint,
                    in_amount: in_amt,
                    out_amount: out_amt,
                    fee_amount: fee_amt,
                    fee_mint: step.swap_info.fee_mint,
                    percent: step.percent,
                }
            })
            .collect();

        Ok(SwapQuote {
            provider: "Jupiter".to_string(),
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            in_amount,
            expected_out_amount,
            planning_min_out_amount,
            fee_bps: self.fee_bps,
            fee_mint: self.fee_mint,
            platform_fee_bps,
            platform_fee_amount,
            router: self.router.unwrap_or_else(|| "iris".to_string()),
            mode: self.mode.unwrap_or_else(|| "ExactIn".to_string()),
            price_impact_pct: self.price_impact_pct,
            route_plan,
            observed_at,
            valid_until: observed_at.saturating_add(ttl_secs),
            is_indicative: true,
        })
    }
}


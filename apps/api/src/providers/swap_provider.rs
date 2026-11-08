use chrono::Utc;
use domain_types::{AssetId, SwapQuote, SwapRouteStep, TokenAmount};
use recovery_engine::{calculate_slippage_bps, calculate_swap_output};

use crate::adapters::jupiter::JupiterSwapQuoteProvider;
use crate::error::ApiError;
use crate::providers::demo_provider::DemoProvider;
use crate::providers::position_provider::BoxFuture;

/// Abstract provider trait for retrieving indicative swap quotes.
pub trait SwapQuoteProvider: Send + Sync {
    fn get_swap_quote<'a>(
        &'a self,
        input_mint: &'a str,
        output_mint: &'a str,
        amount: u64,
        max_slippage_bps: u16,
        refresh: bool,
    ) -> BoxFuture<'a, Result<SwapQuote, ApiError>>;
}

impl SwapQuoteProvider for JupiterSwapQuoteProvider {
    fn get_swap_quote<'a>(
        &'a self,
        input_mint: &'a str,
        output_mint: &'a str,
        amount: u64,
        max_slippage_bps: u16,
        refresh: bool,
    ) -> BoxFuture<'a, Result<SwapQuote, ApiError>> {
        self.get_swap_quote(input_mint, output_mint, amount, max_slippage_bps, refresh)
    }
}

impl SwapQuoteProvider for DemoProvider {
    fn get_swap_quote<'a>(
        &'a self,
        input_mint: &'a str,
        output_mint: &'a str,
        amount: u64,
        max_slippage_bps: u16,
        _refresh: bool,
    ) -> BoxFuture<'a, Result<SwapQuote, ApiError>> {
        Box::pin(async move {
            if amount == 0 {
                return Err(ApiError::BadRequest(
                    "Swap amount must be greater than zero".to_string(),
                ));
            }
            if max_slippage_bps > 10_000 {
                return Err(ApiError::BadRequest(
                    "Max slippage basis points cannot exceed 10,000".to_string(),
                ));
            }

            let pool = self.swap_pool_params();
            let (out_amount, fee_amount) =
                calculate_swap_output(&pool, amount).map_err(ApiError::from)?;

            let slippage_bps = calculate_slippage_bps(&pool, amount, out_amount).unwrap_or(0);

            let planning_min = SwapQuote::compute_planning_min_output(out_amount, max_slippage_bps)
                .map_err(ApiError::from)?;

            let now = Utc::now().timestamp();

            Ok(SwapQuote {
                provider: "DemoPool".to_string(),
                input_mint: input_mint.to_string(),
                output_mint: output_mint.to_string(),
                in_amount: TokenAmount::new(amount, 9, Some(AssetId::SSol)),
                expected_out_amount: TokenAmount::new(out_amount, 6, Some(AssetId::SUsdc)),
                planning_min_out_amount: TokenAmount::new(planning_min, 6, Some(AssetId::SUsdc)),
                fee_bps: pool.fee_bps,
                fee_mint: Some(output_mint.to_string()),
                platform_fee_bps: None,
                platform_fee_amount: None,
                router: "DemoConstantProductPool".to_string(),
                mode: "ExactIn".to_string(),
                price_impact_pct: Some(format!("{:.2}", (slippage_bps as f64) / 100.0)),
                route_plan: vec![SwapRouteStep {
                    amm_key: Some("DemoPoolReserveAccount".to_string()),
                    label: Some("Demo Constant Product Pool".to_string()),
                    input_mint: input_mint.to_string(),
                    output_mint: output_mint.to_string(),
                    in_amount: amount,
                    out_amount,
                    fee_amount,
                    fee_mint: Some(output_mint.to_string()),
                    percent: 100,
                }],
                observed_at: now,
                valid_until: now + 60,
                is_indicative: true,
            })
        })
    }
}

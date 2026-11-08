use std::sync::Arc;

use chrono::Utc;
use domain_types::{
    HealthFactor, ProtocolSource, RecoveryAction, RecoveryPolicy, ALL_ACTIONS_MASK,
};
use recovery_engine::{generate_recovery_candidates, SwapPoolParams};
use risk_engine::compute_health_factor;
use uuid::Uuid;

use crate::dto::{
    RecoveryOptimizeRequest, RecoveryPlanDto, RecoveryQuoteRequest, RecoveryRecommendationResponse,
    ScoreBreakdownDto, SwapQuoteDto, TokenAmountDto,
};
use crate::error::ApiError;
use crate::persistence::RecommendationRepository;
use crate::providers::{OracleProvider, SwapQuoteProvider};
use crate::services::PositionService;

#[derive(Clone)]
pub struct RecoveryService {
    position_service: PositionService,
    oracle_provider: Arc<dyn OracleProvider>,
    swap_quote_provider: Arc<dyn SwapQuoteProvider>,
    swap_pool: SwapPoolParams,
    repository: RecommendationRepository,
}

impl RecoveryService {
    pub fn new(
        position_service: PositionService,
        oracle_provider: Arc<dyn OracleProvider>,
        swap_quote_provider: Arc<dyn SwapQuoteProvider>,
        swap_pool: SwapPoolParams,
        repository: RecommendationRepository,
    ) -> Self {
        Self {
            position_service,
            oracle_provider,
            swap_quote_provider,
            swap_pool,
            repository,
        }
    }

    /// Quotes a single requested recovery action or indicative swap quote.
    pub async fn quote_recovery(
        &self,
        req: RecoveryQuoteRequest,
    ) -> Result<RecoveryPlanDto, ApiError> {
        let max_slip = req.max_slippage_bps.unwrap_or(100);
        let refresh = req.refresh.unwrap_or(false);

        // Standalone swap quote branch (e.g. Jupiter explicit mint pair query)
        if let (Some(input_mint), Some(output_mint), Some(amount_str)) =
            (&req.input_mint, &req.output_mint, &req.amount)
        {
            let amount_raw = amount_str
                .parse::<u64>()
                .map_err(|e| ApiError::BadRequest(format!("Invalid amount string: {}", e)))?;

            let quote = self
                .swap_quote_provider
                .get_swap_quote(input_mint, output_mint, amount_raw, max_slip, refresh)
                .await?;

            let swap_dto = SwapQuoteDto::from_domain(&quote);
            let fee_raw = (quote.expected_out_amount.raw as u128)
                .checked_mul(quote.fee_bps as u128)
                .unwrap_or(0)
                / 10_000;
            let fee_u64 = fee_raw as u64;

            return Ok(RecoveryPlanDto {
                action: "SwapQuote".to_string(),
                action_label: "Indicative Swap Quote".to_string(),
                input_amount: TokenAmountDto::from_domain(&quote.in_amount),
                expected_output_amount: Some(TokenAmountDto::from_domain(
                    &quote.expected_out_amount,
                )),
                min_output_amount: Some(TokenAmountDto::from_domain(
                    &quote.planning_min_out_amount,
                )),
                debt_repaid: TokenAmountDto::from_domain(&quote.expected_out_amount),
                collateral_spent: TokenAmountDto::from_domain(&quote.in_amount),
                collateral_added: TokenAmountDto::zero(quote.in_amount.decimals),
                expected_fee_formatted: format!(
                    "{}.{:06}",
                    fee_u64 / 1_000_000,
                    fee_u64 % 1_000_000
                ),
                expected_fee_raw: fee_u64.to_string(),
                estimated_slippage_bps: max_slip,
                post_health_factor: "1.0000".to_string(),
                post_health_factor_scaled: 10000,
                safety_buffer_bps: 0,
                is_feasible: true,
                rejection_reason: None,
                total_cost_formatted: format!("{}.{:06}", fee_u64 / 1_000_000, fee_u64 % 1_000_000),
                total_cost_raw: fee_u64.to_string(),
                score: 100,
                score_breakdown: ScoreBreakdownDto {
                    capital_efficiency: 100,
                    safety_buffer: 0,
                    fee_penalty: 0,
                    slippage_penalty: 0,
                    total_score: 100,
                },
                swap_quote: Some(swap_dto),
            });
        }

        let pos_id = req.position_id.as_deref().ok_or_else(|| {
            ApiError::BadRequest(
                "Either positionId or (inputMint, outputMint, amount) must be provided".to_string(),
            )
        })?;

        let position = self.position_service.get_position(pos_id).await?;

        // Multi-asset Kamino safety guard:
        // Do not calculate synthetic quotes for KaminoMainnet or multi-asset obligations
        if position.protocol_source == ProtocolSource::KaminoMainnet
            || position.collateral_positions.len() > 1
            || position.debt_positions.len() > 1
        {
            return Err(ApiError::UnprocessableEntity(
                "Recovery quote is unsupported for multi-asset Kamino positions until multi-leg recovery math is implemented"
                    .to_string(),
            ));
        }

        // Standalone swap quote branch (e.g. Jupiter explicit mint pair query)
        if let (Some(input_mint), Some(output_mint), Some(amount_str)) =
            (&req.input_mint, &req.output_mint, &req.amount)
        {
            let amount_raw = amount_str
                .parse::<u64>()
                .map_err(|e| ApiError::BadRequest(format!("Invalid amount string: {}", e)))?;

            let quote = self
                .swap_quote_provider
                .get_swap_quote(input_mint, output_mint, amount_raw, max_slip, refresh)
                .await?;

            let swap_dto = SwapQuoteDto::from_domain(&quote);
            let fee_raw = (quote.expected_out_amount.raw as u128)
                .checked_mul(quote.fee_bps as u128)
                .unwrap_or(0)
                / 10_000;
            let fee_u64 = fee_raw as u64;

            return Ok(RecoveryPlanDto {
                action: "SwapQuote".to_string(),
                action_label: "Indicative Swap Quote".to_string(),
                input_amount: TokenAmountDto::from_domain(&quote.in_amount),
                expected_output_amount: Some(TokenAmountDto::from_domain(
                    &quote.expected_out_amount,
                )),
                min_output_amount: Some(TokenAmountDto::from_domain(
                    &quote.planning_min_out_amount,
                )),
                debt_repaid: TokenAmountDto::from_domain(&quote.expected_out_amount),
                collateral_spent: TokenAmountDto::from_domain(&quote.in_amount),
                collateral_added: TokenAmountDto::zero(quote.in_amount.decimals),
                expected_fee_formatted: format!("{}.{:06}", fee_u64 / 1_000_000, fee_u64 % 1_000_000),
                expected_fee_raw: fee_u64.to_string(),
                estimated_slippage_bps: max_slip,
                post_health_factor: "1.0000".to_string(),
                post_health_factor_scaled: 10000,
                safety_buffer_bps: 0,
                is_feasible: true,
                rejection_reason: None,
                total_cost_formatted: format!("{}.{:06}", fee_u64 / 1_000_000, fee_u64 % 1_000_000),
                total_cost_raw: fee_u64.to_string(),
                score: 100,
                score_breakdown: ScoreBreakdownDto {
                    capital_efficiency: 100,
                    safety_buffer: 0,
                    fee_penalty: 0,
                    slippage_penalty: 0,
                    total_score: 100,
                },
                swap_quote: Some(swap_dto),
            });
        }

        let pos_id = req.position_id.as_deref().ok_or_else(|| {
            ApiError::BadRequest(
                "Either positionId or (inputMint, outputMint, amount) must be provided"
                    .to_string(),
            )
        })?;

        let position = self.position_service.get_position(pos_id).await?;

        // Multi-asset Kamino safety guard:
        // Do not calculate synthetic quotes for KaminoMainnet or multi-asset obligations
        if position.protocol_source == ProtocolSource::KaminoMainnet
            || position.collateral_positions.len() > 1
            || position.debt_positions.len() > 1
        {
            return Err(ApiError::UnprocessableEntity(
                "Recovery quote is unsupported for multi-asset Kamino positions until multi-leg recovery math is implemented"
                    .to_string(),
            ));
        }

        let col_price = self
            .oracle_provider
            .get_price(&position.collateral_asset)
            .await?
            .ok_or_else(|| {
                ApiError::NotFound(format!(
                    "Oracle price not found for collateral asset '{}'",
                    position.collateral_asset
                ))
            })?;

        let debt_price = self
            .oracle_provider
            .get_price(&position.debt_asset)
            .await?
            .ok_or_else(|| {
                ApiError::NotFound(format!(
                    "Oracle price not found for debt asset '{}'",
                    position.debt_asset
                ))
            })?;

        let target_hf = parse_health_factor(req.target_health_factor.as_deref(), 12_500)?;
        let now = Utc::now().timestamp();

        let action_str = req.action.as_deref().unwrap_or("RepayDebt");
        let action = parse_recovery_action(action_str)?;

        // Build policy constraining execution
        let max_debt = req
            .max_debt_repayment
            .as_deref()
            .map(|s| {
                s.parse::<u64>()
                    .map_err(|e| ApiError::BadRequest(e.to_string()))
            })
            .transpose()?
            .unwrap_or(u64::MAX);

        let max_col = req
            .max_collateral_spend
            .as_deref()
            .map(|s| {
                s.parse::<u64>()
                    .map_err(|e| ApiError::BadRequest(e.to_string()))
            })
            .transpose()?
            .unwrap_or(u64::MAX);

        let policy = RecoveryPolicy {
            position: position.id.clone(),
            owner: position.owner.clone(),
            authorized_delegate: None,
            max_debt_repayment: max_debt,
            max_collateral_spend: max_col,
            max_slippage_bps: max_slip,
            min_resulting_hf: target_hf.0,
            allowed_actions: action.bitmask(),
            expiry: now + 3600,
            execution_nonce: 0,
            policy_revision: 1,
            is_active: true,
            schema_version: 2,
        };

        let candidates = generate_recovery_candidates(
            &position,
            &col_price,
            &debt_price,
            target_hf,
            Some(&self.swap_pool),
            Some(&policy),
            now,
        )?;

        let plan = candidates
            .into_iter()
            .find(|c| c.action == action)
            .ok_or_else(|| {
                ApiError::UnprocessableEntity(format!(
                    "Unable to generate viable quote for action '{}'",
                    action_str
                ))
            })?;

        let mut plan_dto = RecoveryPlanDto::from_proposal(&plan);

        // Enrich with swap quote if action is SwapCollateralAndRepay
        if plan.action == RecoveryAction::SwapCollateralAndRepay && plan.collateral_spent.raw > 0 {
            let col_mint = position
                .collateral_positions
                .first()
                .map(|p| p.mint_address.as_str())
                .unwrap_or("So11111111111111111111111111111111111111112");
            let debt_mint = position
                .debt_positions
                .first()
                .map(|p| p.mint_address.as_str())
                .unwrap_or("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

            if let Ok(quote) = self
                .swap_quote_provider
                .get_swap_quote(
                    col_mint,
                    debt_mint,
                    plan.collateral_spent.raw,
                    max_slip,
                    refresh,
                )
                .await
            {
                plan_dto.swap_quote = Some(SwapQuoteDto::from_domain(&quote));
            }
        }

        Ok(plan_dto)
    }

    /// Evaluates and deterministically ranks all recovery options for a position.
    pub async fn optimize_recovery(
        &self,
        req: RecoveryOptimizeRequest,
    ) -> Result<RecoveryRecommendationResponse, ApiError> {
        let position = self.position_service.get_position(&req.position_id).await?;

        // Multi-asset Kamino safety guard:
        // Reject full recovery plan calculation for KaminoMainnet or multi-asset positions
        if position.protocol_source == ProtocolSource::KaminoMainnet
            || position.collateral_positions.len() > 1
            || position.debt_positions.len() > 1
        {
            return Err(ApiError::UnprocessableEntity(
                "Recovery optimization is unsupported for multi-asset Kamino positions until multi-leg recovery math is implemented"
                    .to_string(),
            ));
        }

        let col_price = self
            .oracle_provider
            .get_price(&position.collateral_asset)
            .await?
            .ok_or_else(|| {
                ApiError::NotFound(format!(
                    "Oracle price not found for collateral asset '{}'",
                    position.collateral_asset
                ))
            })?;

        let debt_price = self
            .oracle_provider
            .get_price(&position.debt_asset)
            .await?
            .ok_or_else(|| {
                ApiError::NotFound(format!(
                    "Oracle price not found for debt asset '{}'",
                    position.debt_asset
                ))
            })?;

        let target_hf = parse_health_factor(req.target_health_factor.as_deref(), 12_500)?;
        let now = Utc::now().timestamp();

        let allowed_actions_mask = if let Some(ref actions) = req.allowed_actions {
            let mut mask: u8 = 0;
            for a in actions {
                mask |= parse_recovery_action(a)?.bitmask();
            }
            mask
        } else {
            ALL_ACTIONS_MASK
        };

        let max_debt = req
            .max_debt_repayment
            .as_deref()
            .map(|s| {
                s.parse::<u64>()
                    .map_err(|e| ApiError::BadRequest(e.to_string()))
            })
            .transpose()?
            .unwrap_or(u64::MAX);

        let max_col = req
            .max_collateral_spend
            .as_deref()
            .map(|s| {
                s.parse::<u64>()
                    .map_err(|e| ApiError::BadRequest(e.to_string()))
            })
            .transpose()?
            .unwrap_or(u64::MAX);

        let max_slip = req.max_slippage_bps.unwrap_or(100);

        let policy = RecoveryPolicy {
            position: position.id.clone(),
            owner: position.owner.clone(),
            authorized_delegate: None,
            max_debt_repayment: max_debt,
            max_collateral_spend: max_col,
            max_slippage_bps: max_slip,
            min_resulting_hf: target_hf.0,
            allowed_actions: allowed_actions_mask,
            expiry: now + 3600,
            execution_nonce: 0,
            policy_revision: 1,
            is_active: true,
            schema_version: 2,
        };

        let pre_hf = compute_health_factor(
            &position.collateral_amount,
            &col_price,
            &position.debt_amount,
            &debt_price,
            position.liquidation_threshold_bps,
        )?;

        let candidates = generate_recovery_candidates(
            &position,
            &col_price,
            &debt_price,
            target_hf,
            Some(&self.swap_pool),
            Some(&policy),
            now,
        )?;

        let recovery_id = Uuid::new_v4();
        let expires_at = now + 900; // 15 minute validity

        let recommended_plan = candidates
            .iter()
            .find(|c| c.is_feasible)
            .map(RecoveryPlanDto::from_proposal);

        let alternative_plans: Vec<RecoveryPlanDto> = candidates
            .iter()
            .map(RecoveryPlanDto::from_proposal)
            .collect();

        let response = RecoveryRecommendationResponse {
            recovery_id: recovery_id.to_string(),
            position_id: position.id.clone(),
            status: "Quoted".to_string(),
            pre_health_factor: pre_hf.to_decimal_string(),
            target_health_factor: target_hf.to_decimal_string(),
            recommended_plan,
            alternative_plans,
            disclaimer: "Advisory estimates only. Final recovery plan execution requires user wallet signature and Anchor smart contract policy enforcement on Solana.".to_string(),
            created_at: now,
            expires_at,
        };

        // Persist to repository
        let req_json = serde_json::to_value(&req).unwrap_or_default();
        let resp_json = serde_json::to_value(&response).unwrap_or_default();
        let _ = self
            .repository
            .save_recommendation(
                recovery_id,
                &position.id,
                req_json,
                resp_json,
                "Quoted",
                900,
            )
            .await;

        Ok(response)
    }

    /// Retrieves an existing recovery recommendation by ID.
    pub async fn get_recommendation(
        &self,
        recovery_id: Uuid,
    ) -> Result<RecoveryRecommendationResponse, ApiError> {
        let record = self
            .repository
            .get_recommendation(recovery_id)
            .await?
            .ok_or_else(|| {
                ApiError::NotFound(format!(
                    "Recovery recommendation '{}' not found",
                    recovery_id
                ))
            })?;

        let response: RecoveryRecommendationResponse =
            serde_json::from_value(record.ranked_response)
                .map_err(|e| ApiError::Internal(format!("Deserialization error: {}", e)))?;

        Ok(response)
    }
}

/// Parses a health factor string (e.g. "1.25" or "12500") without floating-point math.
pub fn parse_health_factor(
    input: Option<&str>,
    default_scaled: u64,
) -> Result<HealthFactor, ApiError> {
    let s = match input {
        Some(val) if !val.trim().is_empty() => val.trim(),
        _ => return Ok(HealthFactor(default_scaled)),
    };

    if s.contains('.') {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err(ApiError::BadRequest(
                "Invalid health factor format".to_string(),
            ));
        }
        let whole: u64 = parts[0]
            .parse()
            .map_err(|_| ApiError::BadRequest("Invalid health factor whole number".to_string()))?;

        let frac_str = parts[1];
        let frac_len = frac_str.len();
        if frac_len > 4 {
            return Err(ApiError::BadRequest(
                "Health factor precision exceeds 4 decimal places".to_string(),
            ));
        }

        let frac_num: u64 = frac_str
            .parse()
            .map_err(|_| ApiError::BadRequest("Invalid health factor fraction".to_string()))?;

        let multiplier = 10u64.pow((4 - frac_len) as u32);
        let scaled = whole
            .checked_mul(10_000)
            .and_then(|w| w.checked_add(frac_num * multiplier))
            .ok_or_else(|| ApiError::BadRequest("Health factor overflow".to_string()))?;

        if scaled < 10_000 {
            return Err(ApiError::BadRequest(
                "Target health factor must be at least 1.0 (10,000 scaled)".to_string(),
            ));
        }
        Ok(HealthFactor(scaled))
    } else {
        let scaled: u64 = s
            .parse()
            .map_err(|_| ApiError::BadRequest("Invalid health factor integer".to_string()))?;

        if scaled < 10_000 {
            return Err(ApiError::BadRequest(
                "Target health factor must be at least 1.0 (10,000 scaled)".to_string(),
            ));
        }
        Ok(HealthFactor(scaled))
    }
}

/// Parses an action string into RecoveryAction enum.
fn parse_recovery_action(input: &str) -> Result<RecoveryAction, ApiError> {
    match input.trim().to_ascii_lowercase().as_str() {
        "repaydebt" | "repay_debt" | "repay" => Ok(RecoveryAction::RepayDebt),
        "addcollateral" | "add_collateral" | "collateral" => Ok(RecoveryAction::AddCollateral),
        "swapcollateralandrepay" | "swap_and_repay" | "swap" => {
            Ok(RecoveryAction::SwapCollateralAndRepay)
        }
        other => Err(ApiError::BadRequest(format!(
            "Unsupported recovery action '{}'. Permitted: RepayDebt, AddCollateral, SwapCollateralAndRepay",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_health_factor() {
        assert_eq!(parse_health_factor(Some("1.25"), 10_000).unwrap().0, 12_500);
        assert_eq!(
            parse_health_factor(Some("1.0625"), 10_000).unwrap().0,
            10_625
        );
        assert_eq!(
            parse_health_factor(Some("12500"), 10_000).unwrap().0,
            12_500
        );
        assert!(parse_health_factor(Some("0.95"), 10_000).is_err()); // below 1.0
        assert_eq!(parse_health_factor(None, 12_000).unwrap().0, 12_000);
    }
}

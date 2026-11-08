use domain_types::{
    DomainError, HealthFactor, PositionSnapshot, Price, RecoveryAction, RecoveryPolicy, TokenAmount,
};
use risk_engine::decimal_math::{calculate_quote_value, DEFAULT_QUOTE_DECIMALS};
use risk_engine::health_factor::compute_health_factor;

use crate::candidate::RecoveryProposal;
use crate::feasibility::{check_feasibility, FeasibilityContext};
use crate::ranking::{compute_score, rank_candidates};
use crate::recovery_cost::{
    calculate_slippage_bps, calculate_swap_output, compute_min_output, SwapPoolParams,
};

/// Generates and deterministically ranks recovery proposals for a lending position.
pub fn generate_recovery_candidates(
    position: &PositionSnapshot,
    collateral_price: &Price,
    debt_price: &Price,
    target_hf: HealthFactor,
    pool_params: Option<&SwapPoolParams>,
    policy: Option<&RecoveryPolicy>,
    current_timestamp: i64,
) -> Result<Vec<RecoveryProposal>, DomainError> {
    let mut proposals = Vec::new();

    // 1. Propose RepayDebt
    if let Some(proposal) = propose_repay_debt(
        position,
        collateral_price,
        debt_price,
        target_hf,
        policy,
        current_timestamp,
    )? {
        proposals.push(proposal);
    }

    // 2. Propose AddCollateral
    if let Some(proposal) = propose_add_collateral(
        position,
        collateral_price,
        debt_price,
        target_hf,
        policy,
        current_timestamp,
    )? {
        proposals.push(proposal);
    }

    // 3. Propose SwapCollateralAndRepay (if pool is provided)
    if let Some(pool) = pool_params {
        if let Some(proposal) = propose_swap_and_repay(
            position,
            collateral_price,
            debt_price,
            target_hf,
            pool,
            policy,
            current_timestamp,
        )? {
            proposals.push(proposal);
        }
    }

    // Deterministically score and rank candidates
    rank_candidates(&mut proposals);

    Ok(proposals)
}

/// Computes a RepayDebt candidate proposal.
fn propose_repay_debt(
    position: &PositionSnapshot,
    collateral_price: &Price,
    debt_price: &Price,
    target_hf: HealthFactor,
    policy: Option<&RecoveryPolicy>,
    current_timestamp: i64,
) -> Result<Option<RecoveryProposal>, DomainError> {
    if position.debt_amount.raw == 0 {
        return Ok(None);
    }

    // Current health factor
    let current_hf = compute_health_factor(
        &position.collateral_amount,
        collateral_price,
        &position.debt_amount,
        debt_price,
        position.liquidation_threshold_bps,
    )?;

    if current_hf.0 >= target_hf.0 {
        // Already at or above target HF
        return Ok(None);
    }

    // Effective exponent offset
    let s_c = (collateral_price.exponent as i64)
        .checked_sub(position.collateral_amount.decimals as i64)
        .ok_or(DomainError::MathOverflow)?;
    let s_d = (debt_price.exponent as i64)
        .checked_sub(position.debt_amount.decimals as i64)
        .ok_or(DomainError::MathOverflow)?;

    let mut num = (position.collateral_amount.raw as u128)
        .checked_mul(collateral_price.mantissa as u128)
        .ok_or(DomainError::MathOverflow)?
        .checked_mul(position.liquidation_threshold_bps as u128)
        .ok_or(DomainError::MathOverflow)?;

    let mut unit_den = debt_price.mantissa as u128;

    if s_c > s_d {
        let factor = 10u128
            .checked_pow((s_c - s_d) as u32)
            .ok_or(DomainError::MathOverflow)?;
        num = num.checked_mul(factor).ok_or(DomainError::MathOverflow)?;
    } else if s_d > s_c {
        let factor = 10u128
            .checked_pow((s_d - s_c) as u32)
            .ok_or(DomainError::MathOverflow)?;
        unit_den = unit_den
            .checked_mul(factor)
            .ok_or(DomainError::MathOverflow)?;
    }

    let den = unit_den
        .checked_mul(target_hf.0 as u128)
        .ok_or(DomainError::MathOverflow)?;

    let allowed_debt_u128 = num / den;
    let allowed_debt = allowed_debt_u128 as u64;

    let mut repay_raw = position.debt_amount.raw.saturating_sub(allowed_debt);
    if repay_raw == 0 && position.debt_amount.raw > 0 {
        repay_raw = 1;
    }

    // Ensure resulting HF >= target_hf
    let mut post_hf = compute_health_factor(
        &position.collateral_amount,
        collateral_price,
        &TokenAmount::new(
            position.debt_amount.raw.saturating_sub(repay_raw),
            position.debt_amount.decimals,
            position.debt_amount.asset_id.clone(),
        ),
        debt_price,
        position.liquidation_threshold_bps,
    )?;

    if post_hf.0 < target_hf.0 && repay_raw < position.debt_amount.raw {
        repay_raw = repay_raw.saturating_add(1);
        post_hf = compute_health_factor(
            &position.collateral_amount,
            collateral_price,
            &TokenAmount::new(
                position.debt_amount.raw.saturating_sub(repay_raw),
                position.debt_amount.decimals,
                position.debt_amount.asset_id.clone(),
            ),
            debt_price,
            position.liquidation_threshold_bps,
        )?;
    }

    let debt_repaid = TokenAmount::new(
        repay_raw,
        position.debt_amount.decimals,
        position.debt_amount.asset_id.clone(),
    );
    let collateral_spent = TokenAmount::zero(position.collateral_amount.decimals);
    let collateral_added = TokenAmount::zero(position.collateral_amount.decimals);

    let (is_feasible, rejection_reason) = check_feasibility(&FeasibilityContext {
        action: RecoveryAction::RepayDebt,
        collateral_spent: &collateral_spent,
        debt_repaid: &debt_repaid,
        slippage_bps: 0,
        post_hf,
        available_collateral: &position.collateral_amount,
        current_debt: &position.debt_amount,
        policy,
        current_timestamp,
    });

    let safety_buffer_bps = (post_hf.0 as i64) - (target_hf.0 as i64);
    let (score, score_breakdown) =
        compute_score(RecoveryAction::RepayDebt, post_hf, target_hf, 0, 0);

    let total_cost_quote =
        calculate_quote_value(&debt_repaid, debt_price, DEFAULT_QUOTE_DECIMALS)? as u64;

    Ok(Some(RecoveryProposal {
        action: RecoveryAction::RepayDebt,
        input_amount: debt_repaid.clone(),
        expected_output_amount: None,
        min_output_amount: None,
        debt_repaid,
        collateral_spent,
        collateral_added,
        expected_fee_quote: 0,
        estimated_slippage_bps: 0,
        post_health_factor: post_hf,
        safety_buffer_bps,
        is_feasible,
        rejection_reason,
        total_cost_quote,
        score,
        score_breakdown,
    }))
}

/// Computes an AddCollateral candidate proposal.
fn propose_add_collateral(
    position: &PositionSnapshot,
    collateral_price: &Price,
    debt_price: &Price,
    target_hf: HealthFactor,
    policy: Option<&RecoveryPolicy>,
    current_timestamp: i64,
) -> Result<Option<RecoveryProposal>, DomainError> {
    if position.debt_amount.raw == 0 {
        return Ok(None);
    }

    let current_hf = compute_health_factor(
        &position.collateral_amount,
        collateral_price,
        &position.debt_amount,
        debt_price,
        position.liquidation_threshold_bps,
    )?;

    if current_hf.0 >= target_hf.0 {
        return Ok(None);
    }

    let s_c = (collateral_price.exponent as i64)
        .checked_sub(position.collateral_amount.decimals as i64)
        .ok_or(DomainError::MathOverflow)?;
    let s_d = (debt_price.exponent as i64)
        .checked_sub(position.debt_amount.decimals as i64)
        .ok_or(DomainError::MathOverflow)?;

    let mut num = (target_hf.0 as u128)
        .checked_mul(position.debt_amount.raw as u128)
        .ok_or(DomainError::MathOverflow)?
        .checked_mul(debt_price.mantissa as u128)
        .ok_or(DomainError::MathOverflow)?;

    let mut den = (collateral_price.mantissa as u128)
        .checked_mul(position.liquidation_threshold_bps as u128)
        .ok_or(DomainError::MathOverflow)?;

    if s_d > s_c {
        let factor = 10u128
            .checked_pow((s_d - s_c) as u32)
            .ok_or(DomainError::MathOverflow)?;
        num = num.checked_mul(factor).ok_or(DomainError::MathOverflow)?;
    } else if s_c > s_d {
        let factor = 10u128
            .checked_pow((s_c - s_d) as u32)
            .ok_or(DomainError::MathOverflow)?;
        den = den.checked_mul(factor).ok_or(DomainError::MathOverflow)?;
    }

    // Ceiling division: (num + den - 1) / den
    let needed_collateral_u128 = num
        .checked_add(den.saturating_sub(1))
        .ok_or(DomainError::MathOverflow)?
        .checked_div(den)
        .ok_or(DomainError::DivisionByZero)?;

    let needed_collateral = needed_collateral_u128 as u64;
    let mut add_raw = needed_collateral.saturating_sub(position.collateral_amount.raw);
    if add_raw == 0 {
        add_raw = 1;
    }

    let mut post_hf = compute_health_factor(
        &TokenAmount::new(
            position.collateral_amount.raw.saturating_add(add_raw),
            position.collateral_amount.decimals,
            position.collateral_amount.asset_id.clone(),
        ),
        collateral_price,
        &position.debt_amount,
        debt_price,
        position.liquidation_threshold_bps,
    )?;

    if post_hf.0 < target_hf.0 {
        add_raw = add_raw.saturating_add(1);
        post_hf = compute_health_factor(
            &TokenAmount::new(
                position.collateral_amount.raw.saturating_add(add_raw),
                position.collateral_amount.decimals,
                position.collateral_amount.asset_id.clone(),
            ),
            collateral_price,
            &position.debt_amount,
            debt_price,
            position.liquidation_threshold_bps,
        )?;
    }

    let collateral_added = TokenAmount::new(
        add_raw,
        position.collateral_amount.decimals,
        position.collateral_amount.asset_id.clone(),
    );
    let collateral_spent = TokenAmount::zero(position.collateral_amount.decimals);
    let debt_repaid = TokenAmount::zero(position.debt_amount.decimals);

    let (is_feasible, rejection_reason) = check_feasibility(&FeasibilityContext {
        action: RecoveryAction::AddCollateral,
        collateral_spent: &collateral_spent,
        debt_repaid: &debt_repaid,
        slippage_bps: 0,
        post_hf,
        available_collateral: &position.collateral_amount,
        current_debt: &position.debt_amount,
        policy,
        current_timestamp,
    });

    let safety_buffer_bps = (post_hf.0 as i64) - (target_hf.0 as i64);
    let (score, score_breakdown) =
        compute_score(RecoveryAction::AddCollateral, post_hf, target_hf, 0, 0);

    let total_cost_quote =
        calculate_quote_value(&collateral_added, collateral_price, DEFAULT_QUOTE_DECIMALS)? as u64;

    Ok(Some(RecoveryProposal {
        action: RecoveryAction::AddCollateral,
        input_amount: collateral_added.clone(),
        expected_output_amount: None,
        min_output_amount: None,
        debt_repaid,
        collateral_spent,
        collateral_added,
        expected_fee_quote: 0,
        estimated_slippage_bps: 0,
        post_health_factor: post_hf,
        safety_buffer_bps,
        is_feasible,
        rejection_reason,
        total_cost_quote,
        score,
        score_breakdown,
    }))
}

/// Computes a SwapCollateralAndRepay candidate proposal using integer binary search.
fn propose_swap_and_repay(
    position: &PositionSnapshot,
    collateral_price: &Price,
    debt_price: &Price,
    target_hf: HealthFactor,
    pool_params: &SwapPoolParams,
    policy: Option<&RecoveryPolicy>,
    current_timestamp: i64,
) -> Result<Option<RecoveryProposal>, DomainError> {
    if position.debt_amount.raw == 0 || position.collateral_amount.raw == 0 {
        return Ok(None);
    }

    let current_hf = compute_health_factor(
        &position.collateral_amount,
        collateral_price,
        &position.debt_amount,
        debt_price,
        position.liquidation_threshold_bps,
    )?;

    if current_hf.0 >= target_hf.0 {
        return Ok(None);
    }

    // Binary search over [1, position.collateral_amount.raw] to find minimum collateral to spend
    let mut low: u64 = 1;
    let mut high: u64 = position.collateral_amount.raw.saturating_sub(1);
    if high == 0 {
        return Ok(None);
    }

    let mut optimal_spend: Option<(u64, u64, HealthFactor)> = None;

    while low <= high {
        let mid = low + (high - low) / 2;

        let (out, _) = match calculate_swap_output(pool_params, mid) {
            Ok(res) => res,
            Err(_) => {
                // Out of reserves, search lower
                high = mid.saturating_sub(1);
                continue;
            }
        };

        let repaid = out.min(position.debt_amount.raw);
        let rem_col = position.collateral_amount.raw.saturating_sub(mid);
        let rem_debt = position.debt_amount.raw.saturating_sub(repaid);

        let post_hf = compute_health_factor(
            &TokenAmount::new(
                rem_col,
                position.collateral_amount.decimals,
                position.collateral_amount.asset_id.clone(),
            ),
            collateral_price,
            &TokenAmount::new(
                rem_debt,
                position.debt_amount.decimals,
                position.debt_amount.asset_id.clone(),
            ),
            debt_price,
            position.liquidation_threshold_bps,
        )?;

        if post_hf.0 >= target_hf.0 {
            optimal_spend = Some((mid, repaid, post_hf));
            // Try to find a smaller collateral spend
            high = mid.saturating_sub(1);
        } else {
            // Need more collateral to increase health factor
            low = mid.saturating_add(1);
        }
    }

    let (col_spend_raw, debt_repaid_raw, post_hf) = match optimal_spend {
        Some(res) => res,
        None => {
            // Could not achieve target HF with available collateral
            return Ok(None);
        }
    };

    let (_, fee_raw) = calculate_swap_output(pool_params, col_spend_raw)?;
    let fee_token = TokenAmount::new(
        fee_raw,
        position.collateral_amount.decimals,
        position.collateral_amount.asset_id.clone(),
    );
    let fee_quote =
        calculate_quote_value(&fee_token, collateral_price, DEFAULT_QUOTE_DECIMALS)? as u64;

    let slippage_bps = calculate_slippage_bps(pool_params, col_spend_raw, debt_repaid_raw)?;

    let max_slip = policy.map(|p| p.max_slippage_bps).unwrap_or(100); // 1% default
    let min_out_raw = compute_min_output(debt_repaid_raw, max_slip)?;

    let input_amount = TokenAmount::new(
        col_spend_raw,
        position.collateral_amount.decimals,
        position.collateral_amount.asset_id.clone(),
    );
    let output_amount = TokenAmount::new(
        debt_repaid_raw,
        position.debt_amount.decimals,
        position.debt_amount.asset_id.clone(),
    );
    let min_output = TokenAmount::new(
        min_out_raw,
        position.debt_amount.decimals,
        position.debt_amount.asset_id.clone(),
    );
    let collateral_spent = input_amount.clone();
    let collateral_added = TokenAmount::zero(position.collateral_amount.decimals);
    let debt_repaid = output_amount.clone();

    let (is_feasible, rejection_reason) = check_feasibility(&FeasibilityContext {
        action: RecoveryAction::SwapCollateralAndRepay,
        collateral_spent: &collateral_spent,
        debt_repaid: &debt_repaid,
        slippage_bps,
        post_hf,
        available_collateral: &position.collateral_amount,
        current_debt: &position.debt_amount,
        policy,
        current_timestamp,
    });

    let safety_buffer_bps = (post_hf.0 as i64) - (target_hf.0 as i64);
    let (score, score_breakdown) = compute_score(
        RecoveryAction::SwapCollateralAndRepay,
        post_hf,
        target_hf,
        fee_quote,
        slippage_bps,
    );

    let col_spent_val =
        calculate_quote_value(&collateral_spent, collateral_price, DEFAULT_QUOTE_DECIMALS)? as u64;
    let total_cost_quote = col_spent_val.saturating_add(fee_quote);

    Ok(Some(RecoveryProposal {
        action: RecoveryAction::SwapCollateralAndRepay,
        input_amount,
        expected_output_amount: Some(output_amount),
        min_output_amount: Some(min_output),
        debt_repaid,
        collateral_spent,
        collateral_added,
        expected_fee_quote: fee_quote,
        estimated_slippage_bps: slippage_bps,
        post_health_factor: post_hf,
        safety_buffer_bps,
        is_feasible,
        rejection_reason,
        total_cost_quote,
        score,
        score_breakdown,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::{ACTION_ADD_COLLATERAL, ACTION_REPAY_DEBT, ACTION_SWAP_AND_REPAY};

    #[test]
    fn test_optimizer_golden_scenario_1() {
        // Golden scenario 1: 10 sSOL @ $150 ($1,500), 1,200 sUSDC @ $1.00 ($1,200), LT = 85%
        // Current HF = 1.0625 (10,625)
        // Target HF = 1.2500 (12,500)
        let pos = PositionSnapshot::synthetic_demo("p1", "o1", 10_000_000_000, 1_200_000_000, 8500);
        let col_price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        // Pool has 1,000 SOL and 150,000 USDC reserves, 30 bps fee
        let pool = SwapPoolParams::new(1_000_000_000_000, 150_000_000_000, 30);

        let policy = RecoveryPolicy {
            position: "p1".to_string(),
            owner: "o1".to_string(),
            authorized_delegate: None,
            max_debt_repayment: 1_000_000_000, // max $1,000 repayment
            max_collateral_spend: 5_000_000_000, // max 5 SOL spend
            max_slippage_bps: 100,             // 1%
            min_resulting_hf: 12_500,          // 1.25
            allowed_actions: ACTION_REPAY_DEBT | ACTION_ADD_COLLATERAL | ACTION_SWAP_AND_REPAY,
            expiry: 2000000000,
            nonce: 1,
            is_active: true,
            version: 1,
        };

        let candidates = generate_recovery_candidates(
            &pos,
            &col_price,
            &debt_price,
            HealthFactor(12_500),
            Some(&pool),
            Some(&policy),
            1700000010,
        )
        .unwrap();

        assert_eq!(candidates.len(), 3);
        // All candidates must achieve post_hf >= 12,500
        for c in &candidates {
            assert!(c.post_health_factor.0 >= 12_500);
            assert!(c.is_feasible);
        }

        // Repay debt candidate should repay ~180 USDC (180_000_000)
        let repay_cand = candidates
            .iter()
            .find(|c| c.action == RecoveryAction::RepayDebt)
            .unwrap();
        assert_eq!(repay_cand.debt_repaid.raw, 180_000_000);

        // Add collateral candidate should add ~1.7647 SOL (1_764_705_883)
        let add_cand = candidates
            .iter()
            .find(|c| c.action == RecoveryAction::AddCollateral)
            .unwrap();
        assert_eq!(add_cand.collateral_added.raw, 1_764_705_883);

        // Swap and repay candidate should spend collateral and repay debt with zero external funds
        let swap_cand = candidates
            .iter()
            .find(|c| c.action == RecoveryAction::SwapCollateralAndRepay)
            .unwrap();
        assert!(swap_cand.collateral_spent.raw > 0);
        assert!(swap_cand.debt_repaid.raw > 0);
        assert!(swap_cand.post_health_factor.0 >= 12_500);

        // First ranked candidate under least-cost objective should be RepayDebt ($180.00 < $181.75 < $264.70)
        assert_eq!(candidates[0].action, RecoveryAction::RepayDebt);
    }
}

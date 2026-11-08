use domain_types::{HealthFactor, RecoveryAction, ScoreBreakdown};

use crate::candidate::RecoveryProposal;

/// Computes the detailed score breakdown for a recovery proposal.
pub fn compute_score(
    action: RecoveryAction,
    post_hf: HealthFactor,
    target_hf: HealthFactor,
    fee_quote: u64,
    slippage_bps: u16,
) -> (u32, ScoreBreakdown) {
    // 1. Capital efficiency:
    // SwapCollateralAndRepay requires 0 external funds from wallet -> high score (5000)
    // RepayDebt / AddCollateral requires external wallet capital -> lower base score (3000)
    let capital_efficiency = match action {
        RecoveryAction::SwapCollateralAndRepay => 5_000,
        RecoveryAction::RepayDebt => 3_500,
        RecoveryAction::AddCollateral => 3_000,
    };

    // 2. Safety buffer: bonus for exceeding target HF
    let safety_buffer = if post_hf.0 > target_hf.0 {
        let diff = post_hf.0 - target_hf.0;
        // 100 bps (1%) = 100 points, cap at 2,000 points
        (diff / 10).min(2_000) as u32
    } else {
        0
    };

    // 3. Fee penalty: fee in quote units (e.g. $1 fee in 6 decimals = 1_000_000)
    // 1 USD fee = 10 points penalty
    let fee_penalty = ((fee_quote / 100_000) as u32).min(1_000);

    // 4. Slippage penalty: slippage in bps * 2
    let slippage_penalty = (slippage_bps as u32 * 2).min(2_000);

    let breakdown = ScoreBreakdown {
        capital_efficiency,
        safety_buffer,
        fee_penalty,
        slippage_penalty,
    };

    (breakdown.total_score(), breakdown)
}

/// Deterministically sorts candidate recovery proposals:
/// 1. Feasible candidates come before infeasible candidates.
/// 2. Feasible candidates sorted by score descending (highest score first).
/// 3. Infeasible candidates sorted by score descending.
/// 4. Tie-breaker: action enum discriminant.
pub fn rank_candidates(candidates: &mut [RecoveryProposal]) {
    candidates.sort_by(|a, b| {
        b.is_feasible
            .cmp(&a.is_feasible)
            .then_with(|| b.score.cmp(&a.score))
            .then_with(|| (a.action as u8).cmp(&(b.action as u8)))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::TokenAmount;

    #[test]
    fn test_ranking_prefers_feasible_and_high_score() {
        let mut candidates = vec![
            RecoveryProposal {
                action: RecoveryAction::RepayDebt,
                input_amount: TokenAmount::zero(6),
                expected_output_amount: None,
                min_output_amount: None,
                debt_repaid: TokenAmount::zero(6),
                collateral_spent: TokenAmount::zero(9),
                collateral_added: TokenAmount::zero(9),
                expected_fee_quote: 0,
                estimated_slippage_bps: 0,
                post_health_factor: HealthFactor(12_500),
                safety_buffer_bps: 500,
                is_feasible: true,
                rejection_reason: None,
                score: 3500,
                score_breakdown: ScoreBreakdown::default(),
            },
            RecoveryProposal {
                action: RecoveryAction::SwapCollateralAndRepay,
                input_amount: TokenAmount::zero(9),
                expected_output_amount: None,
                min_output_amount: None,
                debt_repaid: TokenAmount::zero(6),
                collateral_spent: TokenAmount::zero(9),
                collateral_added: TokenAmount::zero(9),
                expected_fee_quote: 0,
                estimated_slippage_bps: 20,
                post_health_factor: HealthFactor(12_500),
                safety_buffer_bps: 500,
                is_feasible: true,
                rejection_reason: None,
                score: 4960,
                score_breakdown: ScoreBreakdown::default(),
            },
            RecoveryProposal {
                action: RecoveryAction::AddCollateral,
                input_amount: TokenAmount::zero(9),
                expected_output_amount: None,
                min_output_amount: None,
                debt_repaid: TokenAmount::zero(6),
                collateral_spent: TokenAmount::zero(9),
                collateral_added: TokenAmount::zero(9),
                expected_fee_quote: 0,
                estimated_slippage_bps: 0,
                post_health_factor: HealthFactor(12_500),
                safety_buffer_bps: 500,
                is_feasible: false,
                rejection_reason: Some("Exceeds max spend".to_string()),
                score: 3000,
                score_breakdown: ScoreBreakdown::default(),
            },
        ];

        rank_candidates(&mut candidates);

        assert_eq!(candidates[0].action, RecoveryAction::SwapCollateralAndRepay);
        assert_eq!(candidates[1].action, RecoveryAction::RepayDebt);
        assert_eq!(candidates[2].action, RecoveryAction::AddCollateral);
    }
}


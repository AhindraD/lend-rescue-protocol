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
    // Capital efficiency score:
    // SwapCollateralAndRepay requires 0 external funds from wallet -> 5,000
    // RepayDebt requires external debt capital -> 3,500
    // AddCollateral requires external collateral capital -> 3,000
    let capital_efficiency = match action {
        RecoveryAction::SwapCollateralAndRepay => 5_000,
        RecoveryAction::RepayDebt => 3_500,
        RecoveryAction::AddCollateral => 3_000,
    };

    // Safety buffer: bonus for exceeding target HF
    let safety_buffer = if post_hf.0 > target_hf.0 {
        let diff = post_hf.0 - target_hf.0;
        // 100 bps (1%) = 100 points, cap at 2,000 points
        (diff / 10).min(2_000) as u32
    } else {
        0
    };

    // Fee penalty: fee in quote units (e.g. $1 fee in 6 decimals = 1_000_000)
    let fee_penalty = ((fee_quote / 100_000) as u32).min(1_000);

    // Slippage penalty: slippage in bps * 2
    let slippage_penalty = (slippage_bps as u32 * 2).min(2_000);

    let breakdown = ScoreBreakdown {
        capital_efficiency,
        safety_buffer,
        fee_penalty,
        slippage_penalty,
    };

    (breakdown.total_score(), breakdown)
}

/// Deterministically sorts candidate recovery proposals according to the least-cost objective:
/// 1. Feasible candidates come before infeasible candidates.
/// 2. Lowest estimated total recovery cost (least-cost viable plan first).
/// 3. Highest post-recovery health factor as a tie-breaker.
/// 4. Lowest estimated slippage.
/// 5. Deterministic action ordering.
pub fn rank_candidates(candidates: &mut [RecoveryProposal]) {
    candidates.sort_by(|a, b| {
        b.is_feasible
            .cmp(&a.is_feasible)
            .then_with(|| a.total_cost_quote.cmp(&b.total_cost_quote))
            .then_with(|| b.post_health_factor.0.cmp(&a.post_health_factor.0))
            .then_with(|| a.estimated_slippage_bps.cmp(&b.estimated_slippage_bps))
            .then_with(|| (a.action as u8).cmp(&(b.action as u8)))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::TokenAmount;

    #[test]
    fn test_ranking_least_cost_objective() {
        let mut candidates = vec![
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
                is_feasible: true,
                rejection_reason: None,
                total_cost_quote: 264_000_000, // $264
                score: 3000,
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
                expected_fee_quote: 540_000,
                estimated_slippage_bps: 20,
                post_health_factor: HealthFactor(12_500),
                safety_buffer_bps: 500,
                is_feasible: true,
                rejection_reason: None,
                total_cost_quote: 181_000_000, // $181
                score: 4960,
                score_breakdown: ScoreBreakdown::default(),
            },
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
                total_cost_quote: 180_000_000, // $180 - Lowest cost!
                score: 3500,
                score_breakdown: ScoreBreakdown::default(),
            },
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
                is_feasible: false,
                rejection_reason: Some("Exceeds max debt".to_string()),
                total_cost_quote: 150_000_000, // Infeasible candidate cannot win
                score: 3500,
                score_breakdown: ScoreBreakdown::default(),
            },
        ];

        rank_candidates(&mut candidates);

        // Feasible candidates come first, sorted by lowest cost:
        // 1. RepayDebt ($180)
        // 2. SwapCollateralAndRepay ($181)
        // 3. AddCollateral ($264)
        // 4. Infeasible RepayDebt
        assert_eq!(candidates[0].action, RecoveryAction::RepayDebt);
        assert!(candidates[0].is_feasible);
        assert_eq!(candidates[0].total_cost_quote, 180_000_000);

        assert_eq!(candidates[1].action, RecoveryAction::SwapCollateralAndRepay);
        assert!(candidates[1].is_feasible);
        assert_eq!(candidates[1].total_cost_quote, 181_000_000);

        assert_eq!(candidates[2].action, RecoveryAction::AddCollateral);
        assert!(candidates[2].is_feasible);

        assert!(!candidates[3].is_feasible);
    }
}

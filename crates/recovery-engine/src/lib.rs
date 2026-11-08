pub mod candidate;
pub mod feasibility;
pub mod optimizer;
pub mod policy;
pub mod ranking;
pub mod recovery_cost;

pub use candidate::RecoveryProposal;
pub use feasibility::{check_feasibility, FeasibilityContext};
pub use optimizer::generate_recovery_candidates;
pub use policy::validate_execution_request;
pub use ranking::{compute_score, rank_candidates};
pub use recovery_cost::{
    calculate_slippage_bps, calculate_swap_output, compute_min_output, SwapPoolParams,
};

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::{
        HealthFactor, PositionSnapshot, Price, RecoveryAction, RecoveryPolicy,
        ACTION_ADD_COLLATERAL, ACTION_REPAY_DEBT, ACTION_SWAP_AND_REPAY,
    };

    #[test]
    fn test_recovery_engine_full_workflow() {
        let pos =
            PositionSnapshot::synthetic_demo("pos1", "owner1", 10_000_000_000, 1_200_000_000, 8500);
        let col_price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        let pool = SwapPoolParams::new(1_000_000_000_000, 150_000_000_000, 30);

        let policy = RecoveryPolicy {
            position: "pos1".to_string(),
            owner: "owner1".to_string(),
            authorized_delegate: Some("delegate1".to_string()),
            max_debt_repayment: 500_000_000,
            max_collateral_spend: 3_000_000_000,
            max_slippage_bps: 100,
            min_resulting_hf: 12_500,
            allowed_actions: ACTION_REPAY_DEBT | ACTION_ADD_COLLATERAL | ACTION_SWAP_AND_REPAY,
            expiry: 2000000000,
            nonce: 1,
            is_active: true,
            version: 1,
        };

        // 1. Validate caller
        assert!(validate_execution_request(
            &policy,
            "delegate1",
            RecoveryAction::RepayDebt,
            0,
            180_000_000,
            0,
            1700000010
        )
        .is_ok());

        // 2. Generate and rank candidates
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

        assert!(!candidates.is_empty());
        assert!(candidates[0].is_feasible);
        assert!(candidates[0].post_health_factor.0 >= 12_500);
    }
}

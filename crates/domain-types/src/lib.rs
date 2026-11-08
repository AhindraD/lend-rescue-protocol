pub mod amount;
pub mod asset;
pub mod error;
pub mod market;
pub mod policy;
pub mod position;
pub mod price;
pub mod recovery;
pub mod risk;

pub use amount::TokenAmount;
pub use asset::{Asset, AssetId};
pub use error::DomainError;
pub use market::MarketConfig;
pub use policy::{
    RecoveryPolicy, ACTION_ADD_COLLATERAL, ACTION_REPAY_DEBT, ACTION_SWAP_AND_REPAY,
    ALL_ACTIONS_MASK,
};
pub use position::{PositionSnapshot, ProtocolSource};
pub use price::Price;
pub use recovery::{RecoveryAction, RecoveryCandidate, ScoreBreakdown};
pub use risk::{HealthFactor, RiskBand, RiskProfile, RiskThresholds};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_amount_arithmetic() {
        let a = TokenAmount::new(1000, 6, Some(AssetId::Usdc));
        let b = TokenAmount::new(500, 6, Some(AssetId::Usdc));
        let sum = a.checked_add(&b).unwrap();
        assert_eq!(sum.raw, 1500);
        let diff = a.checked_sub(&b).unwrap();
        assert_eq!(diff.raw, 500);
    }

    #[test]
    fn test_token_amount_decimal_mismatch() {
        let a = TokenAmount::new(1000, 6, Some(AssetId::Usdc));
        let b = TokenAmount::new(500, 9, Some(AssetId::Sol));
        assert!(a.checked_add(&b).is_err());
    }

    #[test]
    fn test_price_normalization() {
        // $150 with 8 decimals: 150_00000000, exponent = -8
        let price = Price::new(150_00000000, -8, 10_000000, 1700000000, 100);
        assert!(price.is_positive());
        let scaled_8 = price.to_quote_scale(8).unwrap();
        assert_eq!(scaled_8, 150_00000000);
        let scaled_6 = price.to_quote_scale(6).unwrap();
        assert_eq!(scaled_6, 150_000000);
    }

    #[test]
    fn test_health_factor_classification() {
        let hf_healthy = HealthFactor::from_scaled(13_000); // 1.30
        assert_eq!(RiskBand::classify(hf_healthy, None), RiskBand::Healthy);

        let hf_warning = HealthFactor::from_scaled(11_500); // 1.15
        assert_eq!(RiskBand::classify(hf_warning, None), RiskBand::Warning);

        let hf_critical = HealthFactor::from_scaled(10_500); // 1.05
        assert_eq!(RiskBand::classify(hf_critical, None), RiskBand::Critical);

        let hf_liquidatable = HealthFactor::from_scaled(9_500); // 0.95
        assert_eq!(
            RiskBand::classify(hf_liquidatable, None),
            RiskBand::Liquidatable
        );
    }

    #[test]
    fn test_policy_authorization_and_actions() {
        let policy = RecoveryPolicy {
            position: "pos1".to_string(),
            owner: "owner_addr".to_string(),
            authorized_delegate: Some("delegate_addr".to_string()),
            max_debt_repayment: 1_000_000_000,
            max_collateral_spend: 10_000_000_000,
            max_slippage_bps: 100,
            min_resulting_hf: 12_000,
            allowed_actions: ACTION_REPAY_DEBT | ACTION_SWAP_AND_REPAY,
            expiry: 2000000000,
            nonce: 1,
            is_active: true,
            version: 1,
        };

        assert!(policy.is_authorized_caller("owner_addr"));
        assert!(policy.is_authorized_caller("delegate_addr"));
        assert!(!policy.is_authorized_caller("stranger"));

        assert!(policy.is_action_allowed(RecoveryAction::RepayDebt));
        assert!(!policy.is_action_allowed(RecoveryAction::AddCollateral));
        assert!(policy.is_action_allowed(RecoveryAction::SwapCollateralAndRepay));

        assert!(policy.validate_bounds().is_ok());
    }

    #[test]
    fn test_score_breakdown() {
        let score = ScoreBreakdown {
            capital_efficiency: 1000,
            safety_buffer: 500,
            fee_penalty: 50,
            slippage_penalty: 50,
        };
        assert_eq!(score.total_score(), 1400);
    }
}

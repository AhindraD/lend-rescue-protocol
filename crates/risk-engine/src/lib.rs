pub mod decimal_math;
pub mod exposure;
pub mod fixed_point;
pub mod health_factor;
pub mod liquidation_price;
pub mod oracle_validation;
pub mod protocol_calculation;

pub use decimal_math::{
    calculate_quote_value, quote_value_to_token_amount, DEFAULT_QUOTE_DECIMALS,
};
pub use exposure::{
    compute_borrow_capacity, compute_distance_to_liquidation_bps, compute_ltv_bps,
    compute_net_equity,
};
pub use fixed_point::{mul_bps, mul_div, mul_div_ceil, BPS_DIVISOR, HF_PRECISION};
pub use health_factor::{compute_health_factor, is_position_solvent};
pub use liquidation_price::compute_liquidation_price;
pub use oracle_validation::{
    validate_oracle_price, DEFAULT_MAX_CONFIDENCE_BPS, DEFAULT_MAX_ORACLE_AGE_SECS,
};
pub use protocol_calculation::{ProtocolRiskCalculator, SyntheticRiskCalculator};

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::{AssetId, HealthFactor, Price, TokenAmount};

    #[test]
    fn test_risk_engine_integration() {
        let collateral = TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol));
        let col_price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);

        let debt = TokenAmount::new(1_200_000_000, 6, Some(AssetId::SUsdc));
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        let hf = compute_health_factor(&collateral, &col_price, &debt, &debt_price, 8500).unwrap();
        assert_eq!(hf, HealthFactor(10_625));

        let liq_price =
            compute_liquidation_price(&collateral, &debt, &debt_price, -8, 8500).unwrap();
        assert_eq!(liq_price.mantissa, 14_117_647_058);

        let dist = compute_distance_to_liquidation_bps(&col_price, &liq_price).unwrap();
        // Distance from 150 to 141.176 is ~8.823 / 150 = ~5.88% = ~588 bps
        assert!(dist > 580 && dist < 600);
    }
}

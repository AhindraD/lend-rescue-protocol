use domain_types::{DomainError, PositionSnapshot, Price, RiskBand, RiskProfile, RiskThresholds};

use crate::decimal_math::{calculate_quote_value, DEFAULT_QUOTE_DECIMALS};
use crate::exposure::{compute_borrow_capacity, compute_distance_to_liquidation_bps};
use crate::health_factor::compute_health_factor;
use crate::liquidation_price::compute_liquidation_price;
use crate::oracle_validation::validate_oracle_price;

/// Trait defining risk evaluation across different protocol sources (Synthetic, Kamino, etc.).
pub trait ProtocolRiskCalculator {
    fn calculate_risk_profile(
        &self,
        position: &PositionSnapshot,
        collateral_price: &Price,
        debt_price: &Price,
        current_timestamp: i64,
        thresholds: Option<&RiskThresholds>,
    ) -> Result<RiskProfile, DomainError>;
}

/// Risk calculator implementation for Devnet synthetic positions.
#[derive(Debug, Default, Clone, Copy)]
pub struct SyntheticRiskCalculator;

impl ProtocolRiskCalculator for SyntheticRiskCalculator {
    fn calculate_risk_profile(
        &self,
        position: &PositionSnapshot,
        collateral_price: &Price,
        debt_price: &Price,
        current_timestamp: i64,
        thresholds: Option<&RiskThresholds>,
    ) -> Result<RiskProfile, DomainError> {
        // Oracle price validation
        let oracle_valid = validate_oracle_price(collateral_price, current_timestamp, None, None)
            .is_ok()
            && validate_oracle_price(debt_price, current_timestamp, None, None).is_ok();

        if !oracle_valid {
            return Ok(RiskProfile {
                health_factor: None,
                risk_band: RiskBand::Stale,
                liquidation_price: None,
                collateral_value_quote: 0,
                debt_value_quote: 0,
                borrow_capacity_quote: 0,
                distance_to_liquidation_bps: None,
            });
        }

        let collateral_val = calculate_quote_value(
            &position.collateral_amount,
            collateral_price,
            DEFAULT_QUOTE_DECIMALS,
        )?;
        let debt_val =
            calculate_quote_value(&position.debt_amount, debt_price, DEFAULT_QUOTE_DECIMALS)?;

        let hf = compute_health_factor(
            &position.collateral_amount,
            collateral_price,
            &position.debt_amount,
            debt_price,
            position.liquidation_threshold_bps,
        )?;

        let band = RiskBand::classify(hf, thresholds);

        let liq_price = compute_liquidation_price(
            &position.collateral_amount,
            &position.debt_amount,
            debt_price,
            collateral_price.exponent,
            position.liquidation_threshold_bps,
        )?;

        let borrow_cap = compute_borrow_capacity(
            &position.collateral_amount,
            collateral_price,
            &position.debt_amount,
            debt_price,
            position.liquidation_threshold_bps,
            Some(DEFAULT_QUOTE_DECIMALS),
        )?;

        let dist_bps = compute_distance_to_liquidation_bps(collateral_price, &liq_price).ok();

        Ok(RiskProfile {
            health_factor: Some(hf),
            risk_band: band,
            liquidation_price: Some(liq_price),
            collateral_value_quote: collateral_val,
            debt_value_quote: debt_val,
            borrow_capacity_quote: borrow_cap,
            distance_to_liquidation_bps: dist_bps,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::HealthFactor;

    #[test]
    fn test_synthetic_risk_calculator_full_profile() {
        let position =
            PositionSnapshot::synthetic_demo("pos1", "owner1", 10_000_000_000, 1_200_000_000, 8500);
        let col_price = Price::new(150_00000000, -8, 1_00000000, 1700000000, 100);
        let debt_price = Price::new(1_00000000, -8, 1000000, 1700000000, 100);

        let calc = SyntheticRiskCalculator;
        let profile = calc
            .calculate_risk_profile(&position, &col_price, &debt_price, 1700000010, None)
            .unwrap();

        assert_eq!(profile.health_factor, Some(HealthFactor(10_625)));
        assert_eq!(profile.risk_band, RiskBand::Critical); // 1.0625 is Critical (1.00 <= HF < 1.10)
        assert_eq!(profile.collateral_value_quote, 1_500_000_000); // $1,500
        assert_eq!(profile.debt_value_quote, 1_200_000_000); // $1,200
        assert_eq!(profile.borrow_capacity_quote, 75_000_000); // $75
        assert!(profile.distance_to_liquidation_bps.unwrap() > 0);
    }
}

use std::sync::Arc;

use chrono::Utc;
use risk_engine::{ProtocolRiskCalculator, SyntheticRiskCalculator};

use crate::dto::RiskAssessmentResponse;
use crate::error::ApiError;
use crate::providers::OracleProvider;
use crate::services::PositionService;

#[derive(Clone)]
pub struct RiskService {
    position_service: PositionService,
    oracle_provider: Arc<dyn OracleProvider>,
    calculator: SyntheticRiskCalculator,
}

impl RiskService {
    pub fn new(
        position_service: PositionService,
        oracle_provider: Arc<dyn OracleProvider>,
    ) -> Self {
        Self {
            position_service,
            oracle_provider,
            calculator: SyntheticRiskCalculator,
        }
    }

    pub async fn assess_position_risk(
        &self,
        position_id: &str,
    ) -> Result<RiskAssessmentResponse, ApiError> {
        let position = self.position_service.get_position(position_id).await?;

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

        let now = Utc::now().timestamp();

        let profile = self.calculator.calculate_risk_profile(
            &position,
            &col_price,
            &debt_price,
            now,
            None,
        )?;

        let provenance = position.protocol_source.label();

        let proto_dto = position.protocol_metrics.as_ref().map(|m| {
            let format_scaled = |raw: u128| -> String {
                let whole = raw / 1_000_000;
                let frac = raw % 1_000_000;
                format!("{}.{:06}", whole, frac)
            };
            let format_signed = |raw: i128| -> String {
                let sign = if raw < 0 { "-" } else { "" };
                let abs = raw.unsigned_abs();
                let whole = abs / 1_000_000;
                let frac = abs % 1_000_000;
                format!("{}{}.{:06}", sign, whole, frac)
            };
            crate::dto::position::ProtocolRiskMetricsDto {
                borrow_limit_usd: format_scaled(m.borrow_limit_usd_scaled),
                liquidation_threshold_usd: format_scaled(m.liquidation_threshold_usd_scaled),
                total_deposit_usd: format_scaled(m.total_deposit_usd_scaled),
                total_borrow_usd: format_scaled(m.total_borrow_usd_scaled),
                loan_to_value_bps: m.loan_to_value_bps,
                liquidation_threshold_bps: m.liquidation_threshold_bps,
                borrow_utilization_bps: m.borrow_utilization_bps,
                net_account_value_usd: format_signed(m.net_account_value_usd_scaled),
                is_protocol_reported: m.is_protocol_reported,
            }
        });

        Ok(
            RiskAssessmentResponse::from_profile(position.id, &profile, provenance, now)
                .with_protocol_metrics(proto_dto),
        )
    }
}

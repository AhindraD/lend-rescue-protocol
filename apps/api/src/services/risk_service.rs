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

        Ok(RiskAssessmentResponse::from_profile(
            position.id,
            &profile,
            provenance,
            now,
        ))
    }
}

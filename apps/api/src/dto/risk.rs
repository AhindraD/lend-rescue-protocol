use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::common::PriceDto;

/// Evaluated risk assessment output from pure Rust risk-engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RiskAssessmentResponse {
    pub position_id: String,
    pub health_factor: Option<String>,
    pub health_factor_scaled: Option<u64>,
    pub risk_band: String,
    pub risk_band_label: String,
    pub collateral_value_formatted: String,
    pub collateral_value_raw: String,
    pub debt_value_formatted: String,
    pub debt_value_raw: String,
    pub borrow_capacity_formatted: String,
    pub borrow_capacity_raw: String,
    pub liquidation_price: Option<PriceDto>,
    pub distance_to_liquidation_bps: Option<i64>,
    pub data_provenance: String,
    pub evaluated_at: i64,
}

impl RiskAssessmentResponse {
    pub fn from_profile(
        position_id: impl Into<String>,
        profile: &domain_types::RiskProfile,
        provenance: impl Into<String>,
        evaluated_at: i64,
    ) -> Self {
        let hf_str = profile.health_factor.map(|h| h.to_decimal_string());
        let hf_raw = profile.health_factor.map(|h| h.0);

        // Format 6 decimals quote value as human decimal string
        let format_quote = |raw: u128| -> String {
            let whole = raw / 1_000_000;
            let frac = raw % 1_000_000;
            format!("{}.{:06}", whole, frac)
        };

        Self {
            position_id: position_id.into(),
            health_factor: hf_str,
            health_factor_scaled: hf_raw,
            risk_band: format!("{:?}", profile.risk_band),
            risk_band_label: profile.risk_band.label().to_string(),
            collateral_value_formatted: format_quote(profile.collateral_value_quote),
            collateral_value_raw: profile.collateral_value_quote.to_string(),
            debt_value_formatted: format_quote(profile.debt_value_quote),
            debt_value_raw: profile.debt_value_quote.to_string(),
            borrow_capacity_formatted: format_quote(profile.borrow_capacity_quote),
            borrow_capacity_raw: profile.borrow_capacity_quote.to_string(),
            liquidation_price: profile
                .liquidation_price
                .as_ref()
                .map(PriceDto::from_domain),
            distance_to_liquidation_bps: profile.distance_to_liquidation_bps,
            data_provenance: provenance.into(),
            evaluated_at,
        }
    }
}

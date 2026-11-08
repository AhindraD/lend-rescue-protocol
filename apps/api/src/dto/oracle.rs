use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::common::PriceDto;

/// Oracle observation response DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OraclePriceResponse {
    pub asset_id: String,
    pub price: PriceDto,
    pub is_stale: bool,
    pub is_valid: bool,
    pub data_source: String,
    pub age_secs: i64,
}

impl OraclePriceResponse {
    pub fn new(
        asset_id: impl Into<String>,
        price: &domain_types::Price,
        current_timestamp: i64,
        is_valid: bool,
        data_source: impl Into<String>,
    ) -> Self {
        let age_secs = current_timestamp - price.timestamp;
        let is_stale = age_secs > 60;

        Self {
            asset_id: asset_id.into(),
            price: PriceDto::from_domain(price),
            is_stale,
            is_valid,
            data_source: data_source.into(),
            age_secs,
        }
    }
}

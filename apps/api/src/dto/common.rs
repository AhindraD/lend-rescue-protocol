use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Health check response indicating service liveness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}

/// Readiness check response indicating database and dependency connectivity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadyResponse {
    pub ready: bool,
    pub database: String,
    pub demo_mode: bool,
    pub timestamp: i64,
}

/// Standardized JSON error response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: ErrorPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    pub request_id: Option<String>,
}

/// Strongly typed token amount DTO with raw string and human-formatted representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmountDto {
    /// Raw integer base units as string to prevent JavaScript precision loss
    pub raw: String,
    /// Number of decimals for the token
    pub decimals: u8,
    /// Symbol of the asset
    pub symbol: Option<String>,
    /// Exact human-readable decimal representation
    pub formatted: String,
}

impl TokenAmountDto {
    pub fn from_domain(amt: &domain_types::TokenAmount) -> Self {
        Self {
            raw: amt.raw.to_string(),
            decimals: amt.decimals,
            symbol: amt.asset_id.as_ref().map(|a| a.symbol().to_string()),
            formatted: amt.to_decimal_string(),
        }
    }
}

/// Price representation DTO preserving integer precision without floating-point primitives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PriceDto {
    /// Raw price mantissa as string
    pub mantissa: String,
    /// Exponent scaling factor (typically negative, e.g. -8)
    pub exponent: i32,
    /// Absolute confidence interval mantissa as string
    pub confidence: String,
    /// Human formatted price string (e.g. "150.00000000")
    pub formatted: String,
    /// Publication Unix timestamp (seconds)
    pub timestamp: i64,
    /// Publication slot
    pub slot: u64,
}

impl PriceDto {
    pub fn from_domain(price: &domain_types::Price) -> Self {
        Self {
            mantissa: price.mantissa.to_string(),
            exponent: price.exponent,
            confidence: price.confidence.to_string(),
            formatted: price.to_decimal_string(),
            timestamp: price.timestamp,
            slot: price.slot,
        }
    }
}

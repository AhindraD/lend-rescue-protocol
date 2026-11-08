use thiserror::Error;

use crate::error::ApiError;

/// Specific error types for Pyth Core price observation decoding and validation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PythError {
    #[error("Feed ID not found for asset: {0}")]
    MissingFeedMapping(String),

    #[error("Feed ID mismatch: expected {expected}, got {actual}")]
    FeedIdMismatch { expected: String, actual: String },

    #[error("Invalid account owner: expected {expected}, got {actual}")]
    InvalidAccountOwner { expected: String, actual: String },

    #[error("Invalid account data: {0}")]
    InvalidAccountData(String),

    #[error("Non-positive price mantissa: {0}")]
    NonPositivePrice(i64),

    #[error(
        "Confidence interval too wide: confidence {confidence} exceeds threshold for price {price}"
    )]
    ConfidenceTooWide { price: i64, confidence: u64 },

    #[error(
        "Price observation is stale: age {age_secs}s exceeds maximum permitted {max_age_secs}s"
    )]
    StalePrice { age_secs: i64, max_age_secs: i64 },

    #[error("Price publication time is in the future: publish_time {publish_time} > now {now}")]
    FuturePublishTime { publish_time: i64, now: i64 },

    #[error("Invalid or unsupported exponent: {0}")]
    InvalidExponent(i32),

    #[error("Solana RPC transport error: {0}")]
    RpcTransport(String),

    #[error("RPC request timed out after {0}ms")]
    Timeout(u64),

    #[error("Hermes authentication failed: {0}")]
    HermesUnauthorized(String),

    #[error("Hermes service error: {0}")]
    HermesError(String),
}

impl From<PythError> for ApiError {
    fn from(err: PythError) -> Self {
        match &err {
            PythError::MissingFeedMapping(msg) => ApiError::NotFound(msg.clone()),
            PythError::FeedIdMismatch { .. }
            | PythError::InvalidAccountOwner { .. }
            | PythError::InvalidAccountData(_)
            | PythError::NonPositivePrice(_)
            | PythError::ConfidenceTooWide { .. }
            | PythError::StalePrice { .. }
            | PythError::FuturePublishTime { .. }
            | PythError::InvalidExponent(_) => ApiError::UnprocessableEntity(err.to_string()),
            PythError::Timeout(_) => ApiError::Internal(err.to_string()),
            PythError::HermesUnauthorized(msg) => {
                ApiError::BadRequest(format!("Unauthorized: {}", msg))
            }
            PythError::RpcTransport(msg) | PythError::HermesError(msg) => {
                ApiError::Internal(msg.clone())
            }
        }
    }
}

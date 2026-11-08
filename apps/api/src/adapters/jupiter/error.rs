use crate::error::ApiError;

/// Structured errors emitted by Jupiter swap quote operations.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum JupiterError {
    #[error("HTTP request error: {0}")]
    Http(String),

    #[error("Rate limited by Jupiter API. Retry after: {retry_after:?} seconds")]
    RateLimited { retry_after: Option<u64> },

    #[error("Jupiter authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("No route found for requested swap: {0}")]
    NoRouteFound(String),

    #[error("Security violation: unexpected transaction payload received in quote-only request")]
    UnexpectedTransactionPayload,

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid token mint: {0}")]
    InvalidMint(String),

    #[error("Quote is stale: {0}")]
    StaleQuote(String),

    #[error("Request timed out: {0}")]
    Timeout(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<JupiterError> for ApiError {
    fn from(err: JupiterError) -> Self {
        match err {
            JupiterError::RateLimited { retry_after } => {
                let msg = match retry_after {
                    Some(secs) => format!("Jupiter rate limited; retry after {}s", secs),
                    None => "Jupiter rate limited".to_string(),
                };
                ApiError::TooManyRequests(msg)
            }
            JupiterError::AuthenticationFailed(msg) => ApiError::Unauthorized(msg),
            JupiterError::NoRouteFound(msg) => ApiError::UnprocessableEntity(msg),
            JupiterError::UnexpectedTransactionPayload => ApiError::Internal(
                "Security violation: unexpected transaction payload returned in quote-only request"
                    .to_string(),
            ),
            JupiterError::InvalidAmount(msg) | JupiterError::InvalidMint(msg) => {
                ApiError::BadRequest(msg)
            }
            JupiterError::Timeout(msg) => ApiError::Timeout(msg),
            JupiterError::StaleQuote(msg) => ApiError::UnprocessableEntity(msg),
            JupiterError::Http(msg) => ApiError::BadGateway(msg),
            JupiterError::Serialization(msg) => ApiError::Internal(msg),
        }
    }
}


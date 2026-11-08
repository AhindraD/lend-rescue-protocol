use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use domain_types::DomainError;
use thiserror::Error;

pub use crate::dto::common::{ErrorPayload, ErrorResponse};
use crate::middleware::RequestId;

/// Centralized API error representation.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unprocessable request: {0}")]
    UnprocessableEntity(String),

    #[error("Domain calculation error: {0}")]
    Domain(#[from] DomainError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal service error: {0}")]
    Internal(String),

    #[error("Request timed out: {0}")]
    Timeout(String),
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Domain(de) => match de {
                DomainError::MathOverflow | DomainError::DivisionByZero => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                DomainError::InvalidAmount(_)
                | DomainError::InvalidPrice(_)
                | DomainError::DecimalMismatch { .. }
                | DomainError::AssetMismatch { .. } => StatusCode::BAD_REQUEST,
                DomainError::InvalidHealthFactor(_)
                | DomainError::InvalidPolicy(_)
                | DomainError::OracleValidationFailed(_)
                | DomainError::PolicyViolation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            },
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::NotFound(_) => "RESOURCE_NOT_FOUND",
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::UnprocessableEntity(_) => "UNPROCESSABLE_ENTITY",
            ApiError::Domain(de) => match de {
                DomainError::MathOverflow => "MATH_OVERFLOW",
                DomainError::DivisionByZero => "DIVISION_BY_ZERO",
                DomainError::InvalidAmount(_) => "INVALID_AMOUNT",
                DomainError::InvalidPrice(_) => "INVALID_PRICE",
                DomainError::DecimalMismatch { .. } => "DECIMAL_MISMATCH",
                DomainError::AssetMismatch { .. } => "ASSET_MISMATCH",
                DomainError::InvalidHealthFactor(_) => "INVALID_HEALTH_FACTOR",
                DomainError::InvalidPolicy(_) => "INVALID_POLICY",
                DomainError::OracleValidationFailed(_) => "ORACLE_VALIDATION_FAILED",
                DomainError::PolicyViolation(_) => "POLICY_VIOLATION",
            },
            ApiError::Database(_) => "DATABASE_ERROR",
            ApiError::Internal(_) => "INTERNAL_ERROR",
            ApiError::Timeout(_) => "TIMEOUT",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code().to_string();
        let message = match &self {
            // Do not leak sensitive internal database details
            ApiError::Database(_) => "A database error occurred".to_string(),
            ApiError::Internal(_) => "An internal server error occurred".to_string(),
            _ => self.to_string(),
        };

        let body = Json(ErrorResponse {
            error: ErrorPayload {
                code,
                message,
                request_id: None,
            },
        });

        (status, body).into_response()
    }
}

/// Helper to attach request ID to an ApiError response.
pub fn error_response_with_id(
    err: ApiError,
    request_id: &RequestId,
) -> (StatusCode, Json<ErrorResponse>) {
    let status = err.status_code();
    let code = err.error_code().to_string();
    let message = match &err {
        ApiError::Database(_) => "A database error occurred".to_string(),
        ApiError::Internal(_) => "An internal server error occurred".to_string(),
        _ => err.to_string(),
    };

    let body = Json(ErrorResponse {
        error: ErrorPayload {
            code,
            message,
            request_id: Some(request_id.0.clone()),
        },
    });

    (status, body)
}

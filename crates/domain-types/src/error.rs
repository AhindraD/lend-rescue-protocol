use thiserror::Error;

/// Core domain and mathematical errors for LendRescue.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("Arithmetic overflow or underflow occurred")]
    MathOverflow,

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid token amount: {0}")]
    InvalidAmount(&'static str),

    #[error("Invalid price representation: {0}")]
    InvalidPrice(&'static str),

    #[error("Decimal precision mismatch: expected {expected}, got {actual}")]
    DecimalMismatch { expected: u8, actual: u8 },

    #[error("Invalid health factor: {0}")]
    InvalidHealthFactor(&'static str),

    #[error("Invalid recovery policy: {0}")]
    InvalidPolicy(&'static str),

    #[error("Oracle validation failed: {0}")]
    OracleValidationFailed(&'static str),

    #[error("Asset mismatch: expected {expected}, got {actual}")]
    AssetMismatch { expected: String, actual: String },

    #[error("Policy violation: {0}")]
    PolicyViolation(String),
}

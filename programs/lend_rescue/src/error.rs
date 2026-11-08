use anchor_lang::prelude::*;

#[error_code]
pub enum LendRescueError {
    #[msg("Caller is not authorized to perform this operation")]
    UnauthorizedCaller,

    #[msg("Signer is not the authorized oracle update authority")]
    UnauthorizedOracleUpdater,

    #[msg("Signer is not the market authority")]
    UnauthorizedMarketAuthority,

    #[msg("Signer is not the owner of this position")]
    InvalidPositionOwner,

    #[msg("Signer is not the owner of this recovery policy")]
    InvalidPolicyOwner,

    #[msg("Market operations are currently paused")]
    MarketPaused,

    #[msg("Market is already active")]
    MarketAlreadyActive,

    #[msg("Invalid token mint provided for market or operation")]
    InvalidMint,

    #[msg("Token decimals must be positive and within supported range")]
    InvalidDecimals,

    #[msg("Liquidation threshold must be between 1000 bps (10%) and 9900 bps (99%)")]
    InvalidLiquidationThreshold,

    #[msg("Fee basis points cannot exceed 1000 bps (10%)")]
    InvalidFeeBps,

    #[msg("Operation amount must be strictly greater than zero")]
    ZeroAmount,

    #[msg("Insufficient collateral deposited for the requested borrow")]
    InsufficientCollateral,

    #[msg("Borrow would cause position health factor to drop below safe threshold")]
    BorrowLimitExceeded,

    #[msg("Position is unhealthy")]
    PositionUnhealthy,

    #[msg("Position is healthy")]
    PositionHealthy,

    #[msg("Oracle price must be positive and non-zero")]
    InvalidOraclePrice,

    #[msg("Oracle price update is stale and exceeds maximum permitted age")]
    StaleOracle,

    #[msg("Oracle confidence interval exceeds maximum acceptable threshold")]
    OracleConfidenceTooWide,

    #[msg("Oracle feed does not match the asset mint")]
    OracleAssetMismatch,

    #[msg("Recovery policy has been revoked")]
    PolicyRevoked,

    #[msg("Recovery policy has expired")]
    PolicyExpired,

    #[msg("Supplied policy nonce does not match current active policy nonce")]
    InvalidNonce,

    #[msg("Requested recovery action is not permitted by the policy")]
    ActionNotAllowed,

    #[msg("Debt repayment amount exceeds policy maximum limit")]
    ExceedsMaxDebtRepayment,

    #[msg("Collateral expenditure exceeds policy maximum limit")]
    ExceedsMaxCollateralSpend,

    #[msg("Execution slippage exceeds the maximum allowable basis points")]
    SlippageExceeded,

    #[msg("Policy parameters must be strictly bounded (no unlimited limits permitted)")]
    InvalidPolicyBounds,

    #[msg("Allowed actions bitset must specify at least one valid recovery action")]
    InvalidAllowedActions,

    #[msg("Target resulting health factor must be at least 1.0 (10,000 scaled)")]
    TargetHealthFactorTooLow,

    #[msg("Calculated post-recovery health factor fails to satisfy the policy minimum")]
    ResultingHealthFactorTooLow,

    #[msg("Demo swap pool has insufficient reserve liquidity to fulfill swap")]
    InsufficientPoolReserves,

    #[msg("Arithmetic calculation resulted in an overflow or underflow")]
    MathOverflow,

    #[msg("Arithmetic calculation resulted in division by zero")]
    DivisionByZero,
}


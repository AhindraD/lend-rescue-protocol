use domain_types::{HealthFactor, RecoveryAction, ScoreBreakdown, TokenAmount};
use serde::{Deserialize, Serialize};

/// Detailed recovery plan for a single proposed recovery action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryProposal {
    /// Strategy action type
    pub action: RecoveryAction,
    /// Token amount required from wallet or position as input
    pub input_amount: TokenAmount,
    /// Expected token amount output from swap (if applicable)
    pub expected_output_amount: Option<TokenAmount>,
    /// Minimum acceptable token amount output with slippage limit
    pub min_output_amount: Option<TokenAmount>,
    /// Base units of debt repaid
    pub debt_repaid: TokenAmount,
    /// Base units of collateral spent/withdrawn from position
    pub collateral_spent: TokenAmount,
    /// Base units of additional collateral deposited
    pub collateral_added: TokenAmount,
    /// Fee cost in quote currency (6 decimals)
    pub expected_fee_quote: u64,
    /// Estimated slippage in basis points
    pub estimated_slippage_bps: u16,
    /// Expected resulting health factor
    pub post_health_factor: HealthFactor,
    /// Safety buffer in basis points above minimum required HF
    pub safety_buffer_bps: i64,
    /// Whether this proposal satisfies all position and policy constraints
    pub is_feasible: bool,
    /// Detailed reason if not feasible
    pub rejection_reason: Option<String>,
    /// Estimated total cost of recovery in quote currency (e.g. 6 decimals micro-USD)
    pub total_cost_quote: u64,
    /// Total composite optimization score
    pub score: u32,
    /// Breakdown of individual scoring terms
    pub score_breakdown: ScoreBreakdown,
}

use domain_types::{DomainError, RecoveryAction, RecoveryPolicy};

/// Validates an incoming recovery execution request against a recovery policy off-chain.
pub fn validate_execution_request(
    policy: &RecoveryPolicy,
    caller: &str,
    action: RecoveryAction,
    collateral_spent: u64,
    debt_repayment: u64,
    slippage_bps: u16,
    current_timestamp: i64,
) -> Result<(), DomainError> {
    if !policy.is_active {
        return Err(DomainError::PolicyViolation(
            "Recovery policy is inactive".to_string(),
        ));
    }

    if policy.is_expired(current_timestamp) {
        return Err(DomainError::PolicyViolation(
            "Recovery policy has expired".to_string(),
        ));
    }

    if !policy.is_authorized_caller(caller) {
        return Err(DomainError::PolicyViolation(format!(
            "Signer '{}' is not authorized by policy (owner: '{}')",
            caller, policy.owner
        )));
    }

    if !policy.is_action_allowed(action) {
        return Err(DomainError::PolicyViolation(format!(
            "Action '{}' is not allowed by policy bitset",
            action
        )));
    }

    if debt_repayment > policy.max_debt_repayment {
        return Err(DomainError::PolicyViolation(format!(
            "Debt repayment ({}) exceeds max policy limit ({})",
            debt_repayment, policy.max_debt_repayment
        )));
    }

    if collateral_spent > policy.max_collateral_spend {
        return Err(DomainError::PolicyViolation(format!(
            "Collateral expenditure ({}) exceeds max policy limit ({})",
            collateral_spent, policy.max_collateral_spend
        )));
    }

    if slippage_bps > policy.max_slippage_bps {
        return Err(DomainError::PolicyViolation(format!(
            "Slippage ({} bps) exceeds max policy limit ({} bps)",
            slippage_bps, policy.max_slippage_bps
        )));
    }

    Ok(())
}

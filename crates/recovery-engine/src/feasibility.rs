use domain_types::{HealthFactor, RecoveryAction, RecoveryPolicy, TokenAmount};

/// Parameters for evaluating recovery proposal feasibility.
#[derive(Debug, Clone)]
pub struct FeasibilityContext<'a> {
    pub action: RecoveryAction,
    pub collateral_spent: &'a TokenAmount,
    pub debt_repaid: &'a TokenAmount,
    pub slippage_bps: u16,
    pub post_hf: HealthFactor,
    pub available_collateral: &'a TokenAmount,
    pub current_debt: &'a TokenAmount,
    pub policy: Option<&'a RecoveryPolicy>,
    pub current_timestamp: i64,
}

/// Checks if a proposed recovery satisfies all policy and position constraints.
pub fn check_feasibility(ctx: &FeasibilityContext) -> (bool, Option<String>) {
    // 1. Basic position sanity checks
    if ctx.current_debt.raw == 0 {
        return (false, Some("Position has no outstanding debt".to_string()));
    }

    if ctx.collateral_spent.raw > ctx.available_collateral.raw {
        return (
            false,
            Some(format!(
                "Required collateral expenditure ({}) exceeds available position collateral ({})",
                ctx.collateral_spent.raw, ctx.available_collateral.raw
            )),
        );
    }

    // 2. Policy-specific checks
    if let Some(p) = ctx.policy {
        if !p.is_active {
            return (
                false,
                Some("Recovery policy is inactive or revoked".to_string()),
            );
        }

        if p.is_expired(ctx.current_timestamp) {
            return (
                false,
                Some(format!(
                    "Recovery policy expired at timestamp {} (current: {})",
                    p.expiry, ctx.current_timestamp
                )),
            );
        }

        if !p.is_action_allowed(ctx.action) {
            return (
                false,
                Some(format!(
                    "Recovery action '{}' is not permitted by policy",
                    ctx.action
                )),
            );
        }

        if ctx.debt_repaid.raw > p.max_debt_repayment {
            return (
                false,
                Some(format!(
                    "Debt repayment ({}) exceeds policy limit ({})",
                    ctx.debt_repaid.raw, p.max_debt_repayment
                )),
            );
        }

        if ctx.collateral_spent.raw > p.max_collateral_spend {
            return (
                false,
                Some(format!(
                    "Collateral spend ({}) exceeds policy limit ({})",
                    ctx.collateral_spent.raw, p.max_collateral_spend
                )),
            );
        }

        if ctx.slippage_bps > p.max_slippage_bps {
            return (
                false,
                Some(format!(
                    "Estimated slippage ({} bps) exceeds policy limit ({} bps)",
                    ctx.slippage_bps, p.max_slippage_bps
                )),
            );
        }

        if ctx.post_hf.0 < p.min_resulting_hf {
            return (
                false,
                Some(format!(
                    "Post-recovery health factor ({}) fails to achieve policy minimum ({})",
                    ctx.post_hf,
                    HealthFactor(p.min_resulting_hf)
                )),
            );
        }
    } else {
        // Without an explicit policy, minimum requirement is solvency (HF >= 1.0000 = 10,000)
        if !ctx.post_hf.is_solvent() {
            return (
                false,
                Some(format!(
                    "Post-recovery health factor ({}) is below solvency threshold (1.0000)",
                    ctx.post_hf
                )),
            );
        }
    }

    (true, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_types::{AssetId, ACTION_REPAY_DEBT, ACTION_SWAP_AND_REPAY};

    #[test]
    fn test_feasibility_exceeds_max_debt() {
        let policy = RecoveryPolicy {
            position: "pos".to_string(),
            owner: "owner".to_string(),
            authorized_delegate: None,
            max_debt_repayment: 500_000_000,
            max_collateral_spend: 10_000_000_000,
            max_slippage_bps: 100,
            min_resulting_hf: 12_000,
            allowed_actions: ACTION_REPAY_DEBT | ACTION_SWAP_AND_REPAY,
            expiry: 2000000000,
            nonce: 1,
            is_active: true,
            version: 1,
        };

        let col_spent = TokenAmount::zero(9);
        let debt_repaid = TokenAmount::new(600_000_000, 6, Some(AssetId::SUsdc)); // Exceeds 500_000_000
        let avail_col = TokenAmount::new(10_000_000_000, 9, Some(AssetId::SSol));
        let cur_debt = TokenAmount::new(1_200_000_000, 6, Some(AssetId::SUsdc));

        let ctx = FeasibilityContext {
            action: RecoveryAction::RepayDebt,
            collateral_spent: &col_spent,
            debt_repaid: &debt_repaid,
            slippage_bps: 0,
            post_hf: HealthFactor(12_500),
            available_collateral: &avail_col,
            current_debt: &cur_debt,
            policy: Some(&policy),
            current_timestamp: 1700000000,
        };

        let (feasible, reason) = check_feasibility(&ctx);

        assert!(!feasible);
        assert!(reason.unwrap().contains("exceeds policy limit"));
    }
}

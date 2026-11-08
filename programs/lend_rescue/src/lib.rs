#![allow(unexpected_cfgs)]
#![allow(clippy::too_many_arguments)]

use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("FDH6cF9xvoTBpJiPgAEciVcNueEqhLYxXQcPDpM6PpUV");

#[program]
pub mod lend_rescue {
    use super::*;

    /// Initializes a synthetic lending market with collateral and debt mint parameters
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        market_id: u64,
        liquidation_threshold_bps: u16,
        demo_swap_fee_bps: u16,
    ) -> Result<()> {
        let bump = ctx.bumps.market;
        ctx.accounts.initialize_market(
            market_id,
            liquidation_threshold_bps,
            demo_swap_fee_bps,
            bump,
        )
    }

    /// Initializes a mock oracle feed for an asset in this market
    pub fn initialize_mock_oracle(
        ctx: Context<InitializeMockOracle>,
        initial_price: i64,
        exponent: i32,
        confidence: u64,
    ) -> Result<()> {
        let bump = ctx.bumps.oracle;
        ctx.accounts
            .initialize_mock_oracle(initial_price, exponent, confidence, bump)
    }

    /// Updates the mock oracle price mantissa and confidence (update authority only)
    pub fn update_mock_price(
        ctx: Context<UpdateMockPrice>,
        new_price: i64,
        confidence: u64,
    ) -> Result<()> {
        ctx.accounts.update_mock_price(new_price, confidence)
    }

    /// Creates a user lending position in the synthetic market
    pub fn create_demo_position(ctx: Context<CreateDemoPosition>, position_id: u64) -> Result<()> {
        let bump = ctx.bumps.position;
        ctx.accounts.create_demo_position(position_id, bump)
    }

    /// Deposits collateral tokens into the market collateral vault
    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        ctx.accounts.deposit_collateral(amount)
    }

    /// Mints synthetic debt tokens against deposited collateral
    pub fn borrow_demo_debt(ctx: Context<BorrowDemoDebt>, amount: u64) -> Result<()> {
        ctx.accounts.borrow_demo_debt(amount)
    }

    /// Initializes a deterministic demo swap pool for synthetic route execution
    pub fn initialize_demo_swap_pool(
        ctx: Context<InitializeDemoSwapPool>,
        fee_bps: u16,
    ) -> Result<()> {
        let bump = ctx.bumps.pool;
        ctx.accounts.initialize_demo_swap_pool(fee_bps, bump)
    }

    /// Creates an on-chain recovery policy constraining future rescue transactions
    pub fn create_recovery_policy(
        ctx: Context<CreateRecoveryPolicy>,
        authorized_delegate: Option<Pubkey>,
        max_debt_repayment: u64,
        max_collateral_spend: u64,
        max_slippage_bps: u16,
        min_resulting_hf: u64,
        allowed_actions: u8,
        expiry: i64,
    ) -> Result<()> {
        let bump = ctx.bumps.policy;
        ctx.accounts.create_recovery_policy(
            CreateRecoveryPolicyArgs {
                authorized_delegate,
                max_debt_repayment,
                max_collateral_spend,
                max_slippage_bps,
                min_resulting_hf,
                allowed_actions,
                expiry,
            },
            bump,
        )
    }

    /// Updates recovery policy parameters and increments the policy revision
    pub fn update_recovery_policy(
        ctx: Context<UpdateRecoveryPolicy>,
        new_delegate: Option<Pubkey>,
        new_max_debt_repayment: u64,
        new_max_collateral_spend: u64,
        new_max_slippage_bps: u16,
        new_min_resulting_hf: u64,
        new_allowed_actions: u8,
        new_expiry: i64,
    ) -> Result<()> {
        ctx.accounts
            .update_recovery_policy(UpdateRecoveryPolicyArgs {
                new_delegate,
                new_max_debt_repayment,
                new_max_collateral_spend,
                new_max_slippage_bps,
                new_min_resulting_hf,
                new_allowed_actions,
                new_expiry,
            })
    }

    /// Immediately revokes a recovery policy and increments the policy revision
    pub fn revoke_recovery_policy(ctx: Context<RevokeRecoveryPolicy>) -> Result<()> {
        ctx.accounts.revoke_recovery_policy()
    }

    /// Executes an authorized recovery action within strict on-chain policy bounds
    pub fn execute_recovery(
        ctx: Context<ExecuteRecovery>,
        args: ExecuteRecoveryArgs,
    ) -> Result<()> {
        let receipt_bump = ctx.bumps.receipt;
        ctx.accounts.execute_recovery(args, receipt_bump)
    }

    /// Emergency pause/unpause switch for market authority
    pub fn pause_market(ctx: Context<PauseMarket>, is_paused: bool) -> Result<()> {
        ctx.accounts.pause_market(is_paused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use state::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_account_init_space_and_discriminator() {
        // Assert every account struct defines an INIT_SPACE constant
        assert!(MarketConfig::INIT_SPACE > 0);
        assert!(MockOracle::INIT_SPACE > 0);
        assert!(Position::INIT_SPACE > 0);
        assert!(DemoSwapPool::INIT_SPACE > 0);
        assert!(RecoveryReceipt::INIT_SPACE > 0);
        assert!(RecoveryPolicy::INIT_SPACE > 0);

        // Verify total allocated space includes the 8-byte Anchor discriminator
        assert_eq!(
            8 + MarketConfig::INIT_SPACE,
            8 + 32 + 8 + 32 + 32 + 32 + 1 + 1 + 32 + 32 + 2 + 2 + 1 + 1 + 1
        );
        assert_eq!(
            8 + MockOracle::INIT_SPACE,
            8 + 32 + 32 + 8 + 4 + 8 + 8 + 8 + 32 + 1 + 1
        );
        assert_eq!(
            8 + Position::INIT_SPACE,
            8 + 32 + 32 + 8 + 8 + 8 + 32 + 32 + 1 + 1 + 1
        );
        assert_eq!(
            8 + DemoSwapPool::INIT_SPACE,
            8 + 32 + 32 + 32 + 32 + 32 + 2 + 32 + 1 + 1
        );
        assert_eq!(
            8 + RecoveryReceipt::INIT_SPACE,
            8 + 32 + 32 + 32 + 32 + 1 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 1
        );
    }

    #[test]
    fn test_recovery_policy_init_space_option_bounds() {
        // Option<Pubkey> InitSpace is 1 (tag) + 32 (Pubkey) = 33 bytes
        let dummy_policy_none = RecoveryPolicy {
            position: Pubkey::default(),
            owner: Pubkey::default(),
            authorized_delegate: None,
            max_debt_repayment: 100,
            max_collateral_spend: 200,
            max_slippage_bps: 100,
            min_resulting_hf: 11_000,
            allowed_actions: 7,
            expiry: 1000,
            execution_nonce: 0,
            policy_revision: 1,
            is_active: true,
            schema_version: RECOVERY_POLICY_SCHEMA_VERSION,
            bump: 254,
        };

        let dummy_policy_some = RecoveryPolicy {
            authorized_delegate: Some(Pubkey::new_unique()),
            ..dummy_policy_none
        };

        let mut buf_none = Vec::new();
        dummy_policy_none.try_serialize(&mut buf_none).unwrap();
        // Serialized buffer includes 8-byte discriminator + struct fields
        assert!(buf_none.len() <= 8 + RecoveryPolicy::INIT_SPACE);

        let mut buf_some = Vec::new();
        dummy_policy_some.try_serialize(&mut buf_some).unwrap();
        assert_eq!(buf_some.len(), 8 + RecoveryPolicy::INIT_SPACE);
    }

    #[test]
    fn test_policy_authorization_semantics() {
        let owner = Pubkey::new_unique();
        let delegate = Pubkey::new_unique();
        let attacker = Pubkey::new_unique();

        // 1. None: owner-only
        let policy_none = RecoveryPolicy {
            position: Pubkey::default(),
            owner,
            authorized_delegate: None,
            max_debt_repayment: 100,
            max_collateral_spend: 200,
            max_slippage_bps: 100,
            min_resulting_hf: 11_000,
            allowed_actions: 7,
            expiry: 1000,
            execution_nonce: 0,
            policy_revision: 1,
            is_active: true,
            schema_version: RECOVERY_POLICY_SCHEMA_VERSION,
            bump: 254,
        };

        assert!(policy_none.is_authorized_caller(&owner));
        assert!(!policy_none.is_authorized_caller(&delegate));
        assert!(!policy_none.is_authorized_caller(&attacker));

        // 2. Some(delegate): owner and delegate both authorized
        let policy_some = RecoveryPolicy {
            authorized_delegate: Some(delegate),
            ..policy_none
        };

        assert!(policy_some.is_authorized_caller(&owner));
        assert!(policy_some.is_authorized_caller(&delegate));
        assert!(!policy_some.is_authorized_caller(&attacker));
    }
}

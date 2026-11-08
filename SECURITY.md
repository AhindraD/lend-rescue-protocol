# LendRescue Security Policy & Threat Model (`SECURITY.md`)

LendRescue is designed with a defense-in-depth security architecture. Because the system interacts with financial lending positions and collateral, strict security controls and invariant enforcement are required at every layer.

---

## 1. Security Principles

1. **Non-Custodial Architecture**:
   - LendRescue never custodies user private keys, seed phrases, or wallet credentials.
   - All transactions (market interactions, policy creation, policy revocation, and recovery execution) require cryptographic signatures from the user's connected wallet or an authorized delegate.
   - The backend API is stateless with respect to signing authority; it cannot sign or broadcast transactions on behalf of users.

2. **On-Chain Policy Invariants as Primary Defense**:
   - The on-chain Anchor smart contract (`programs/lend_rescue`) is the ultimate arbiter of safety.
   - The contract treats backend recommendations, quotes, and off-chain data as untrusted advisory inputs.
   - All policy constraints (spending limits, slippage caps, minimum post-recovery Health Factor, expiry, and nonces) are validated independently on-chain.

3. **Atomic Execution & State Rollback**:
   - If any policy constraint, oracle check, or token balance check fails, the entire transaction reverts.
   - Failed recovery attempts leave position collateral, debt liabilities, wallet balances, and policy nonces completely untouched.

---

## 2. Threat Modeling & Mitigations

| Threat | Description | Mitigation in LendRescue |
|---|---|---|
| **Malicious Backend Proposal** | A compromised or buggy API proposes an unsafe recovery action (e.g. excessive collateral spend or high slippage). | On-chain Anchor program independently validates all parameters against `RecoveryPolicy`. If spend exceeds limits or post-HF is below minimum, the contract reverts. |
| **Replay Attack** | An attacker intercepts an old, valid recovery transaction signature and replays it later when market conditions have changed. | Every policy enforces a monotonically increasing `nonce`. Successful execution consumes the nonce, permanently invalidating previous transaction payloads. |
| **Oracle Manipulation / Flash Crash** | An attacker manipulates an oracle price feed or supplies stale/zero prices to trigger an unwarranted recovery or bypass safety checks. | Oracles require authorized feed IDs, valid positive mantissas, bounded confidence intervals ($\le 2\%$), and freshness checks ($age \le 60\text{s}$). Stale/wide data aborts execution. |
| **Unauthorized Execution** | A malicious third party attempts to invoke `execute_recovery` to liquidate or alter a victim's position. | `execute_recovery` strictly requires caller to be `policy.owner` or `policy.authorized_delegate`. Other callers are rejected with `UnauthorizedCaller`. |
| **Slippage Exploitation / MEV** | Sandwich attacks or frontrunning on the swap component drain user collateral. | Policy enforces `max_slippage_bps`. Swaps through `DemoSwapPool` use deterministic math and verify minimum output tokens before proceeding. |
| **Accidental Mainnet Broadcast** | A user or developer accidentally submits a synthetic recovery transaction to Solana Mainnet. | Hardcoded environment isolation: seed scripts and execution endpoints reject Mainnet cluster configurations. Mainnet operations are strictly read-only. |
| **Emergency Market Distress** | An unforeseen contract anomaly or pool imbalance occurs. | Market authority retains an emergency `pause_market` instruction to halt all synthetic debt creation and recovery executions immediately. |

---

## 3. On-Chain Invariant Checklist

Before executing any token transfer or state update, `execute_recovery` enforces:

```rust
// 1. Market must not be paused
require!(!market_config.is_paused, RecoveryError::MarketPaused);

// 2. Caller must be owner or authorized active delegate
require!(
    ctx.accounts.authority.key() == policy.owner
    || ctx.accounts.authority.key() == policy.authorized_delegate,
    RecoveryError::UnauthorizedCaller
);

// 3. Policy must be active and not expired
require!(policy.is_active, RecoveryError::PolicyRevoked);
require!(Clock::get()?.unix_timestamp <= policy.expiry, RecoveryError::PolicyExpired);

// 4. Action must be permitted in the policy's allowed_actions bitset
require!(policy.is_action_allowed(action), RecoveryError::ActionNotAllowed);

// 5. Execution nonce must match current policy nonce
require!(args.nonce == policy.nonce, RecoveryError::InvalidNonce);

// 6. Oracle must be fresh and within confidence tolerance
require!(oracle.is_fresh(Clock::get()?.unix_timestamp, MAX_ORACLE_AGE), RecoveryError::StaleOracle);
require!(oracle.confidence <= MAX_CONFIDENCE_THRESHOLD, RecoveryError::ConfidenceTooWide);

// 7. Spending amounts must be within explicit caps
require!(args.debt_repayment <= policy.max_debt_repayment, RecoveryError::ExceedsMaxRepayment);
require!(args.collateral_spend <= policy.max_collateral_spend, RecoveryError::ExceedsMaxCollateralSpend);
require!(args.slippage_bps <= policy.max_slippage_bps, RecoveryError::SlippageExceeded);
```

Following token operations:
```rust
// 8. Re-compute synthetic Health Factor from actual post-state balances
let post_hf = position.compute_synthetic_hf(&oracle, &market_config)?;
require!(post_hf >= policy.min_resulting_hf, RecoveryError::ResultingHealthFactorTooLow);

// 9. Consume nonce and write immutable receipt
policy.nonce = policy.nonce.checked_add(1).ok_or(RecoveryError::MathOverflow)?;
```

---

## 4. Responsible Disclosure

If you discover a vulnerability within LendRescue smart contracts, backend services, or frontend applications, please report it responsibly:

- **Email**: `security@lendrescue.internal` (or file a private security advisory on GitHub).
- **Guidelines**:
  - Do not disclose the issue publicly until a fix has been validated.
  - Provide detailed reproduction steps, proof-of-concept transactions, or code snippets.
  - Allow reasonable time for remediation before any public disclosure.

*LendRescue is an open-source research and hackathon project. Code audits must be completed prior to any consideration of real-capital Mainnet deployment.*

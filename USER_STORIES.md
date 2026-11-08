# LendRescue User Stories (`USER_STORIES.md`)

This document outlines the user stories, acceptance criteria, and adversarial/edge case scenarios for the LendRescue product across its key actors and workflows.

---

## Story 1: Wallet Connection & Cluster Selection
- **As a** DeFi borrower,
- **I want to** connect my Solana wallet (Phantom, Solflare, etc.) and choose between Devnet, Mainnet Observation, and Local Surfpool environments,
- **So that** I can interact with my positions in the appropriate operational mode.

### Acceptance Criteria:
- **Given** an unauthenticated visitor on the web app,
- **When** the user clicks "Connect Wallet" and authorizes their wallet provider,
- **Then** the UI displays the truncated wallet address and active cluster (`Solana Devnet` by default for execution).
- **Given** the wallet is connected to Devnet,
- **When** the user switches cluster to "Mainnet Observation",
- **Then** all action buttons (Execute, Create Policy) update to show `Read-Only / Mainnet Observed`, disabling transaction broadcasts.

---

## Story 2: Position Risk Assessment
- **As a** borrower with an active debt position,
- **I want to** see an immediate, clear breakdown of my collateral, debt, current Health Factor ($HF$), and liquidation distance,
- **So that** I know exactly how vulnerable my position is to market liquidation.

### Acceptance Criteria:
- **Given** a synthetic position with 10 `sSOL` collateral and 1,200 `sUSDC` debt,
- **When** the `sSOL` oracle price is \$150 and liquidation threshold is 85%,
- **Then** the UI displays $HF = 1.0625$ with a visual badge `Warning` and explicit numerical distance to liquidation (\$141.18 liquidation price).
- **Given** any position display,
- **Then** risk levels must be indicated by text and numeric values, never by color alone.

---

## Story 3: Oracle Provenance & Freshness Inspection
- **As a** risk-conscious trader,
- **I want to** inspect the exact price feed source, update timestamp, and confidence interval for all assets,
- **So that** I do not make recovery decisions based on stale or untrusted market data.

### Acceptance Criteria:
- **Given** an asset detail card,
- **When** the oracle feed published an update within the last 30 seconds with confidence interval $< 1\%$,
- **Then** the UI displays a green freshness status with exact slot/time and feed source.
- **Given** an oracle price whose age exceeds 60 seconds or confidence interval exceeds 2%,
- **Then** the status flags `Stale / Unreliable`, and the recovery optimizer refuses to generate an executable recommendation.

---

## Story 4: Price Shock Simulation in Risk Lab
- **As a** user assessing downside market scenarios,
- **I want to** simulate a sudden 10% to 30% drop in collateral price,
- **So that** I can observe the resulting Health Factor decline and preview necessary recovery actions before market moves happen.

### Acceptance Criteria:
- **Given** a position with $HF = 1.0625$ at \$150 `sSOL`,
- **When** the user adjusts the Risk Lab slider to simulate a drop to \$120,
- **Then** the simulated $HF$ dynamically recalculates to $0.85$ (Liquidatable), clearly marked with a `Simulated / Shock` banner.
- **Given** a connected Devnet test wallet,
- **When** the user clicks "Apply Shock to Devnet Mock Oracle",
- **Then** an on-chain `update_mock_price` transaction is signed and submitted, updating the actual on-chain mock oracle price to \$120.

---

## Story 5: Recovery Option Comparison
- **As a** borrower facing an at-risk position,
- **I want to** compare Repay Debt, Add Collateral, and Swap-and-Repay strategies side-by-side,
- **So that** I can pick the strategy that minimizes capital cost and slippage while achieving my target safety margin.

### Acceptance Criteria:
- **Given** an at-risk position ($HF = 1.0625$),
- **When** the user opens the Recovery Lab,
- **Then** the engine presents three ranked candidates:
  1. *Repay Debt*: Requires 200 `sUSDC`, 0% slippage, post-$HF = 1.275$.
  2. *Add Collateral*: Requires 2.0 `sSOL` (\$300 value), post-$HF = 1.275$.
  3. *Swap Collateral & Repay*: Sells 1.8 `sSOL` via DemoSwapPool with 0.3% fee and 0.5% slippage to repay 265 `sUSDC`, achieving post-$HF = 1.150$ with zero external liquid funds required.
- **Given** an infeasible candidate (e.g. user has insufficient token balance or pool has insufficient reserves),
- **Then** that candidate is displayed as disabled with an explicit rejection reason.

---

## Story 6: Bounded Recovery Policy Creation
- **As a** position owner,
- **I want to** create an on-chain recovery policy that establishes strict spending caps, slippage limits, and minimum target $HF$,
- **So that** neither an execution error nor an unauthorized party can execute a transaction outside my explicit risk parameters.

### Acceptance Criteria:
- **Given** an active position without a policy,
- **When** the owner submits `create_recovery_policy` with `max_debt_repayment = 300`, `max_collateral_spend = 2.5`, `max_slippage = 100 bps` (1%), `min_resulting_hf = 1.20`, and `expiry = 1 hour`,
- **Then** a `RecoveryPolicy` PDA is initialized with `nonce = 1` and `is_active = true`.
- **Given** an attempt to create a policy with zero or unconstrained limits,
- **Then** client validation and Anchor smart contract constraints reject the transaction.

---

## Story 7: Immediate Policy Revocation
- **As a** position owner,
- **I want to** revoke my recovery policy at any time with a single click,
- **So that** I can instantly invalidate all outstanding delegate authorizations and pre-signed recovery plans.

### Acceptance Criteria:
- **Given** an active recovery policy,
- **When** the owner signs a `revoke_recovery_policy` instruction,
- **Then** the on-chain account updates `is_active = false`.
- **Given** a revoked policy,
- **When** any party attempts to call `execute_recovery`,
- **Then** the transaction immediately fails with `RecoveryError::PolicyRevoked`.

---

## Story 8: Successful On-Chain Recovery Execution
- **As a** position owner or authorized delegate,
- **I want to** execute an approved recovery transaction on Solana Devnet,
- **So that** my position’s Health Factor is restored to a safe level and an immutable receipt is produced.

### Acceptance Criteria:
- **Given** an active policy and a position with $HF = 1.0625$,
- **When** the caller submits `execute_recovery` for `RepayDebt` with 200 `sUSDC` within policy bounds,
- **Then** the transaction succeeds on Devnet, position debt decreases by 200 `sUSDC`, post-$HF$ recalculates to $\ge 1.25$, policy `nonce` increments to 2, and a `RecoveryReceipt` PDA is recorded.

---

## Story 9: Out-of-Policy Recovery Rejection (Atomicity Verification)
- **As a** security-focused user,
- **I want to** know that if any execution parameter breaches my policy, the transaction fails completely with zero state change,
- **So that** my funds are never partially transferred or left in a damaged state.

### Acceptance Criteria:
- **Given** a policy requiring `min_resulting_hf = 1.25` and `max_debt_repayment = 100`,
- **When** a transaction attempts to repay 150 `sUSDC` (violating `max_debt_repayment`) OR a swap delivers insufficient output such that post-$HF = 1.15$ (violating `min_resulting_hf`),
- **Then** the transaction aborts with an on-chain program error.
- **Then** account inspection proves collateral balance, debt balance, wallet tokens, and policy nonces are 100% identical to pre-transaction values.

---

## Story 10: On-Chain Recovery Receipt Inspection
- **As an** auditor or user,
- **I want to** view the permanent on-chain `RecoveryReceipt` for an executed rescue,
- **So that** I have cryptographic proof of the action taken, amounts spent, and resulting safety metrics.

### Acceptance Criteria:
- **Given** an executed recovery transaction,
- **When** the user navigates to `/activity`,
- **Then** the UI fetches the `RecoveryReceipt` PDA and displays the executed action, consumed nonce, pre-HF, post-HF, execution slot, and block explorer link.

---

## Story 11: Transparent Provenance & State Labels
- **As a** user navigating the console,
- **I want to** distinguish at all times whether data is from real Mainnet, an indicative quote, a local fork, or synthetic Devnet,
- **So that** I am never misled into believing a simulation moved real assets.

### Acceptance Criteria:
- **Given** any metric or screen,
- **Then** one of five badges is prominently displayed:
  - `Mainnet Observed`: Live Kamino/Pyth read-only data.
  - `Quote Only`: Indicative Jupiter DEX quote.
  - `Surfpool Simulated`: Local fork sandbox simulation.
  - `Devnet Executed`: Transaction confirmed on Solana Devnet.
  - `Demo / Synthetic`: Synthetic mock market or Risk Lab shock.

---

## Story 12: Accessible & Resilient Landing Page
- **As a** user on a low-powered mobile device or with motion sensitivity,
- **I want to** browse the landing page smoothly without heavy WebGL lockups or disorienting animations,
- **So that** I can understand the product easily.

### Acceptance Criteria:
- **Given** a user with `prefers-reduced-motion` enabled or WebGL disabled,
- **Then** the 3D hero canvas is replaced with an elegant static SVG/CSS risk instrument graphic.
- **Given** standard browsing,
- **Then** the Three.js canvas dynamically loads only on the client, targets a maximum DPR of 1.5, and pauses rendering whenever scrolled out of viewport.

---

## 13. Adversarial & Edge Case Stories

| ID | Scenario | Expected System Behavior |
|---|---|---|
| **ADV-1** | Replay of previous transaction with consumed nonce | Reverts with `RecoveryError::InvalidNonce`. |
| **ADV-2** | Call `execute_recovery` after policy expiry timestamp | Reverts with `RecoveryError::PolicyExpired`. |
| **ADV-3** | Unauthorized third-party signs execution | Reverts with `RecoveryError::UnauthorizedCaller`. |
| **ADV-4** | Mock oracle price set to zero or negative | Reverts with `OracleError::InvalidPrice`. |
| **ADV-5** | Oracle price confidence interval $> 5\%$ | Reverts with `OracleError::ConfidenceTooWide`. |
| **ADV-6** | Swap execution slippage exceeds `max_slippage_bps` | Reverts with `RecoveryError::SlippageExceeded`. |
| **ADV-7** | Market authority activates emergency pause | All state-changing instructions revert with `MarketError::MarketPaused`. |

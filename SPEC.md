# LendRescue Specification (`SPEC.md`)

## 1. Product Overview and Objectives

**LendRescue** is a DeFi lending-position risk monitoring and recovery orchestration system on Solana. Its core purpose is to find the safest, most cost-effective recovery route for collateralized borrowing positions before liquidation occurs, and to enforce that recovery on-chain under owner-defined policy constraints.

### 1.1 The Problem
In decentralized lending protocols (such as Kamino, Solend/Save, Marginfi), borrowers deposit collateral to borrow debt assets. When collateral prices drop or borrow interest accrues, the position's Health Factor ($HF$) declines toward the liquidation threshold ($HF \le 1.0$). At that threshold, third-party liquidators seize collateral at a steep liquidation discount (often 5% to 15%), inflicting permanent capital loss on the borrower and potentially creating bad debt for the protocol.

Existing alert tools and dashboards inform users that their position is at risk, but leave them with critical pain points:
1. **Decision Paralysis under Pressure**: Borrowers must manually calculate whether to repay debt with wallet funds, post additional collateral, or swap a portion of existing collateral to extinguish debt.
2. **Execution Blindness**: Estimating slippage, DEX fees, and oracle confidence intervals during volatile markets is complex; an uncalibrated swap can worsen position health rather than rescue it.
3. **Custodial Risk in Automation**: Existing automated keeper services demand full custody or blanket signing access over borrower wallets, exposing users to catastrophic smart contract or operational exploits.

### 1.2 The Objective
LendRescue resolves this dilemma through a three-stage pipeline:
1. **Observe**: Ingest real-time lending positions, protocol risk parameters, oracle feeds (Pyth / mock), and swap routes (Jupiter / synthetic).
2. **Decide**: A deterministic, pure-Rust financial engine evaluates and ranks feasible recovery strategies by capital efficiency, fees, price impact, and resulting safety margin.
3. **Enforce**: An Anchor smart contract independently validates owner-configured recovery policies (maximum collateral spend, maximum debt repayment, minimum resulting $HF$, expiry, and nonces) and executes the approved token CPIs atomically on Solana Devnet.

---

## 2. Target Users and Use Cases

### 2.1 Target Audiences
- **Active DeFi Borrowers**: Individuals with leveraged positions seeking automated or guided risk mitigation without granting private key custody.
- **Yield & Loop Farmers**: Users operating collateralized leverage strategies requiring calibrated partial deleveraging during market drawdowns.
- **DAO Treasuries & Protocol Operators**: Institutional desks managing protocol collateral requiring auditable on-chain limits for risk mitigation.

### 2.2 Key Use Cases
- **Guided Manual Rescue**: User observes position nearing warning threshold ($HF < 1.15$), simulates price shocks in the Risk Lab, reviews ranked recovery candidates, approves an on-chain policy, and executes the recovery via their connected wallet.
- **Delegated Policy Rescue**: User delegates recovery execution rights to an authorized agent or keeper, restricted by an immutable on-chain policy with strict spending caps and minimum target $HF$.
- **Synthetic Price Shock Testing**: Developer or user tests how a 20% oracle price drop impacts a position and verifies that an out-of-policy transaction reverts cleanly without state mutation.

---

## 3. Functional Requirements

### 3.1 Market & Position Observation (FR-1)
- **FR-1.1**: Read lending position snapshots including collateral mint, collateral base units, debt mint, debt base units, and protocol source.
- **FR-1.2**: Support read-only observation of Kamino lending obligations on Solana Mainnet.
- **FR-1.3**: Support full synthetic position lifecycle (create, deposit collateral, borrow debt) on Solana Devnet.

### 3.2 Pure Rust Financial & Risk Engine (FR-2)
- **FR-2.1**: Calculate synthetic Health Factor using checked fixed-point arithmetic:
  $$\text{HF} = \frac{\text{Collateral Value} \times \text{Liquidation Threshold (bps)}}{10\,000 \times \text{Debt Value}}$$
- **FR-2.2**: Calculate liquidation price for the collateral asset in debt currency terms.
- **FR-2.3**: Classify positions into deterministic risk bands:
  - `Healthy`: $HF \ge 1.25$
  - `Warning`: $1.10 \le HF < 1.25$
  - `Critical`: $1.00 \le HF < 1.10$
  - `Liquidatable`: $HF < 1.00$
  - `Stale` / `Unavailable`: Oracle timestamp expired or data missing.
- **FR-2.4**: Prohibit floating-point primitives (`f32`, `f64`) in all authoritative financial and risk calculations.

### 3.3 Oracle Validation (FR-3)
- **FR-3.1**: Ingest price snapshots carrying price mantissa, scale/exponent, confidence interval, publish timestamp/slot, and feed source.
- **FR-3.2**: Reject oracle prices where confidence interval exceeds configured proportion (e.g. $> 2\%$ of price).
- **FR-3.3**: Reject stale oracle prices where publication age exceeds maximum allowed duration (e.g. 60 seconds).
- **FR-3.4**: For Devnet demonstration, provide a dedicated `MockOracle` account controlled by an authorized update key.

### 3.4 Recovery Optimization & Candidate Ranking (FR-4)
- **FR-4.1**: Compare three standard recovery actions:
  1. `RepayDebt`: Transfer debt tokens from wallet to repay liability.
  2. `AddCollateral`: Transfer additional collateral tokens from wallet to boost collateral balance.
  3. `SwapCollateralAndRepay`: Withdraw permitted collateral, swap through liquidity pool, and repay debt with output tokens.
- **FR-4.2**: Generate candidate profiles detailing required input tokens/amounts, expected output tokens/amounts, protocol and swap fees, indicative slippage, and estimated post-recovery $HF$.
- **FR-4.3**: Rank candidates by capital efficiency and safety score, providing explicit rejection reasons for invalid or infeasible options.

### 3.5 On-Chain Recovery Policy Management (FR-5)
- **FR-5.1**: Position owner can initialize a `RecoveryPolicy` PDA bound to their position.
- **FR-5.2**: Policy must enforce explicit constraints:
  - `authorized_delegate`: Specific public key or owner-only constraint (`Pubkey::default()`).
  - `max_debt_repayment`: Upper bound in base units.
  - `max_collateral_spend`: Upper bound in base units.
  - `max_slippage_bps`: Maximum allowed swap slippage in basis points.
  - `min_resulting_hf`: Minimum acceptable post-recovery Health Factor (scaled integer).
  - `allowed_actions`: Bitset or enum restricting permitted operations.
  - `expiry`: Absolute Unix timestamp or slot threshold.
  - `nonce`: Monotonically increasing execution nonce.
  - `is_active`: Boolean status flag.
- **FR-5.3**: Owner can update policy parameters, automatically invalidating existing pre-signed plans by bumping the nonce.
- **FR-5.4**: Owner can revoke policy instantly at any time.

### 3.6 On-Chain Execution & Enforcement (FR-6)
- **FR-6.1**: The Anchor program instruction `execute_recovery` must independently enforce all policy bounds, oracle validity, token account ownership, and PDA derivations before executing token movements.
- **FR-6.2**: Recalculate post-recovery synthetic $HF$ on-chain from actual updated balances. If the calculated post-$HF$ fails to meet `min_resulting_hf`, the entire transaction reverts.
- **FR-6.3**: On successful execution, consume the current policy nonce, record an immutable `RecoveryReceipt` account, and emit a structured execution event.
- **FR-6.4**: On instruction failure, all token transfers and account states must revert atomically to their exact pre-transaction state.

---

## 4. Non-Functional Requirements

### 4.1 Correctness & Determinism
- Core domain engine must produce identical mathematical results for identical input structs across any platform (x86_64, aarch64, SBPF).
- Zero division, overflow, and underflow conditions must be handled gracefully through typed errors (`DomainError::MathOverflow`, etc.).
- Rounding direction must be explicit: safety thresholds round conservatively against the user.

### 4.2 Security & Trust Boundaries
- **Zero Custody**: LendRescue never requests, stores, or transmits user private keys.
- **Zero Trust in Backend Calculation**: The Anchor program never accepts backend-asserted post-$HF$ numbers; it derives state strictly from on-chain accounts and SPL Token CPI results.
- **Replay Protection**: Nonce consumption ensures a signed recovery transaction payload cannot be executed twice.
- **Emergency Circuit Breaker**: Market authority can pause synthetic market operations in case of detected anomalies.

### 4.3 Performance & Resource Efficiency
- Landing page interactive 3D hero must be dynamically imported, client-rendered only, capped at 1.5 device pixel ratio, and paused when off-screen via `IntersectionObserver`.
- Axum REST API endpoints must achieve $< 50\text{ms}$ latency for risk and recovery evaluation using local in-memory calculation.
- Anchor instructions must operate well within standard Solana compute unit limits (max 200,000 CU per execution).

### 4.4 Accessibility & UX Integrity
- Adhere to WCAG 2.1 Level AA standards: color is never the exclusive indicator of position risk; clear numerical and text badges accompany all status indicators.
- Respect `prefers-reduced-motion` media queries by providing static SVG/CSS visual fallbacks for 3D elements.

---

## 5. Scope & Environment Matrix

### 5.1 Environment Distinction

| Environment | Purpose | Permitted Operations | Provenance Label |
|---|---|---|---|
| **Solana Mainnet** | Real-world position observation | Read-only queries of Kamino reserves and Pyth feeds | `Mainnet Observed` |
| **Solana Devnet** | End-to-end recovery demonstration | Synthetic market creation, mock oracle shocks, wallet-signed policy execution | `Devnet Executed` |
| **Local Fork (Surfpool)** | Mainnet account inspection & instruction simulation | Local sandbox simulation of protocol state; strictly no external broadcast | `Surfpool Simulated` |
| **Indicative Aggregators** | Third-party swap routing | Price quotation and route discovery; not guaranteed fills | `Quote Only` |
| **Mock Engine** | Risk Lab simulation | Controlled price manipulation and parameter adjustment | `Demo / Synthetic` |

### 5.2 Explicit MVP Non-Goals
1. Creating a general-purpose decentralized lending protocol.
2. Building an AMM or DEX liquidity pool for production trading.
3. Operating an autonomous liquidation bot or keeper network with discretionary signing authority.
4. Broadcasting state-changing transactions to Solana Mainnet.
5. Custodying user funds or private key material in any form.
6. Re-implementing the internal account structures or private code of Kamino.

---

## 6. Acceptance Criteria

1. **Phase 0 (Foundation)**: Root documentation, toolchain lockfiles, workspace manifests, and Anthropic `frontend-design` skill established and verified.
2. **Phase 1 (Anchor Program)**: Modular Anchor program compiles cleanly under Anchor 1.2.2 and Solana CLI 2.1.0 with zero warnings.
3. **Phase 2 (Anchor Tests)**: TypeScript integration tests prove 100% pass rate on happy path recoveries and complete atomic rollback on failure scenarios.
4. **Phase 3 (Rust Engine)**: Pure-Rust risk engine passes all unit tests, property tests, and Clippy linting with synthetic golden fixtures.
5. **Phase 4 (Backend API)**: Axum API serves documented endpoints, orchestrates engine calculations, and emits provenance-tagged JSON DTOs.
6. **Phase 5 (Adapters)**: Read-only Kamino adapter, Pyth/Mock oracle adapter, Jupiter quote adapter, and Surfpool simulator pass integration tests.
7. **Phase 6 (Web Application)**: Next.js console passes typecheck, linting, tests, and production build, featuring an editorial design system and lazy 3D hero.
8. **Phase 7 (End-to-End Demo)**: Repeatable Devnet demo executes a successful recovery, creates an on-chain `RecoveryReceipt`, and proves that an out-of-policy transaction reverts with untouched balances.

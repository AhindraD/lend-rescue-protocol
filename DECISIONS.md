# Architecture Decision Records (`DECISIONS.md`)

This document records the architectural and design decisions made for LendRescue, following the standard ADR format: Context, Decision, Consequences, and Status.

---

## Table of Decisions

- [ADR-001: Synthetic-First MVP Scope for Devnet Recovery](#adr-001-synthetic-first-mvp-scope-for-devnet-recovery)
- [ADR-002: On-Chain Anchor Enforcement vs. Advisory Backend Recommendation](#adr-002-on-chain-anchor-enforcement-vs-advisory-backend-recommendation)
- [ADR-003: Pure-Rust Domain Engine Decoupled from Backend and Database](#adr-003-pure-rust-domain-engine-decoupled-from-backend-and-database)
- [ADR-004: Strict Checked Fixed-Point Arithmetic for Financial Logic](#adr-004-strict-checked-fixed-point-arithmetic-for-financial-logic)
- [ADR-005: Pinning the `@solana/web3.js` v1 Stack for Anchor 1.2.2 Compatibility](#adr-005-pinning-the-solanaweb3js-v1-stack-for-anchor-122-compatibility)
- [ADR-006: Environment Isolation (Mainnet Read-Only, Devnet Executable, Surfpool Simulation)](#adr-006-environment-isolation-mainnet-read-only-devnet-executable-surfpool-simulation)
- [ADR-007: Lightweight, Isolated, Lazy-Loaded Three.js Landing Hero](#adr-007-lightweight-isolated-lazy-loaded-threejs-landing-hero)
- [ADR-008: Adoption of Anthropic `frontend-design` Agent Skill](#adr-008-adoption-of-anthropic-frontend-design-agent-skill)
- [ADR-009: Pinned Toolchain Alignment (Agave 4.1.2, Platform Tools v1.57) and SBPF Stack Optimization](#adr-009-pinned-toolchain-alignment-agave-412-platform-tools-v157-and-sbpf-stack-optimization)
- [ADR-010: Axum REST API Architecture, Provider Abstraction, and Least-Cost Ranking Invariant](#adr-010-axum-rest-api-architecture-provider-abstraction-and-least-cost-ranking-invariant)
- [ADR-011: Anchor Architecture Hardening: InitSpace Sizing, Context Method Standardization, and Explicit Authorization/Nonce Semantics](#adr-011-anchor-architecture-hardening-initspace-sizing-context-method-standardization-and-explicit-authorizationnonce-semantics)
- [ADR-012: Pyth Core Persistent On-Chain Oracle Integration, Strict Provenance Metadata, and Zero Silent Fallback](#adr-012-pyth-core-persistent-on-chain-oracle-integration-strict-provenance-metadata-and-zero-silent-fallback)

---

### ADR-001: Synthetic-First MVP Scope for Devnet Recovery

- **Status**: Accepted
- **Context**: Attempting to implement an end-to-end recovery mechanism directly against live Mainnet lending protocols (like Kamino) creates severe risks: unpredictable third-party DEX liquidity on testnets, potential loss of user funds, high testing friction, and tight coupling to proprietary internal account layouts.
- **Decision**: Build a self-contained synthetic lending market on Solana Devnet featuring:
  - Controlled synthetic SPL tokens (`sSOL` collateral, `sUSDC` debt).
  - A deterministic demo swap pool (`DemoSwapPool`).
  - An authorized mock oracle (`MockOracle`) that can be deterministically price-shocked.
  - Read-only observation of real Mainnet Kamino accounts for telemetry.
- **Consequences**:
  - *Positive*: Complete, repeatable testability in CI and local validator environments without relying on external liquidity.
  - *Positive*: Clear separation between verified demonstration code and unverified external protocol assumptions.
  - *Negative*: Real Mainnet position unwinding cannot execute directly in the MVP; it is limited to observation and Devnet demonstration.

---

### ADR-002: On-Chain Anchor Enforcement vs. Advisory Backend Recommendation

- **Status**: Accepted
- **Context**: Many DeFi automation architectures compute parameters off-chain and pass pre-approved metrics to the smart contract, relying on the contract to blindly trust backend outputs. If the backend is compromised or experiences bugs, user funds can be drained.
- **Decision**: Treat the backend API as purely advisory. The on-chain Anchor program (`lend_rescue`) independently validates owner policies:
  - Re-reads post-execution token balances directly from SPL token accounts.
  - Recalculates synthetic Health Factor on-chain using integer math.
  - Reverts the entire transaction if the resulting Health Factor fails to meet `policy.min_resulting_hf` or if spending caps are exceeded.
- **Consequences**:
  - *Positive*: Non-custodial security guarantee. Users are protected even if the backend proposes a malformed or compromised plan.
  - *Positive*: Transaction atomicity: failed conditions leave user balances and position states untouched.
  - *Negative*: Slightly higher Solana compute unit (CU) consumption for on-chain state derivation and checks.

---

### ADR-003: Pure-Rust Domain Engine Decoupled from Backend and Database

- **Status**: Accepted
- **Context**: Embedding financial risk formulas into web controllers (Axum/Express) or SQL stored procedures makes unit testing difficult, harms reusability, and introduces non-deterministic environment dependencies.
- **Decision**: Implement all risk calculations, liquidation formulas, oracle validations, and recovery rankings in pure-Rust crates (`crates/domain-types`, `crates/risk-engine`, `crates/recovery-engine`) before writing the backend API. These crates have zero dependencies on Axum, SQLx, HTTP clients, Anchor macros, or Solana RPC.
- **Consequences**:
  - *Positive*: Deterministic, blazingly fast calculation modules testable via standard `cargo test` and property testing (`proptest`).
  - *Positive*: Engine logic can be compiled into CLI tools, WebAssembly, or backend servers without code duplication.
  - *Negative*: Requires explicit domain DTO translation at the API adapter boundary.

---

### ADR-004: Strict Checked Fixed-Point Arithmetic for Financial Logic

- **Status**: Accepted
- **Context**: Standard floating-point numbers (`f32`, `f64`) introduce rounding discrepancies, IEEE 754 precision loss, and non-deterministic behavior across hardware architectures, which is unacceptable for financial balances and liquidation checks.
- **Decision**: Strictly prohibit `f32` and `f64` in domain and on-chain logic. Use integer base units (`u64`, `u128`) and a documented fixed-point representation with basis points (bps) for ratios. All mathematical operations must use checked or saturating arithmetic.
- **Consequences**:
  - *Positive*: Eliminates silent underflow/overflow bugs and floating-point non-determinism.
  - *Positive*: Guarantees that arithmetic invariants hold identically across Rust and SBPF.
  - *Negative*: Requires rigorous normalization of token decimals (e.g. 9 decimals for SOL vs 6 decimals for USDC) before value comparisons.

---

### ADR-005: Pinning the `@solana/web3.js` v1 Stack for Anchor 1.2.2 Compatibility

- **Status**: Accepted
- **Context**: Solana recently introduced `@solana/web3.js` v2/v3 (Solana Kit). However, Anchor 1.2.2's official client `@anchor-lang/core` is strictly compatible with legacy `@solana/web3.js` v1 (specifically `1.99.0`) and `@solana/spl-token` `0.4.x`. Mixing v2/v3 into the Anchor TypeScript path results in unresolvable peer dependency conflicts and broken IDL decoders.
- **Decision**: Pin `@solana/web3.js` at `1.99.0`, `@solana/spl-token` at `0.4.15`, and `@anchor-lang/core` at `1.2.2` under TypeScript `5.9.3` for all Anchor test and client execution paths. Do not upgrade web3.js in the Anchor workspace.
- **Consequences**:
  - *Positive*: Rock-solid Anchor integration test execution without IDL deserialization failures.
  - *Positive*: Full alignment with official Anchor 1.2.x documentation.
  - *Negative*: Does not leverage the new functional pipeline API of web3.js v2 for smart contract tests.

---

### ADR-006: Environment Isolation (Mainnet Read-Only, Devnet Executable, Surfpool Simulation)

- **Status**: Accepted
- **Context**: Users and operators must never be confused about whether a transaction moved real capital or was executed in a sandbox.
- **Decision**: Enforce strict environment isolation across all UI screens and API endpoints:
  1. **Mainnet**: Strictly read-only observation. No wallet signing or broadcasting routes exist for Mainnet.
  2. **Devnet**: The only environment where state-changing recovery transactions can be signed and submitted.
  3. **Surfpool**: Local fork environment used for inspecting Mainnet state and simulating instructions locally without broadcasting.
  4. Every data point and UI card carries a visible provenance badge (`Mainnet Observed`, `Quote Only`, `Surfpool Simulated`, `Devnet Executed`, `Demo / Synthetic`).
- **Consequences**:
  - *Positive*: Eliminates the risk of accidental Mainnet transaction broadcast.
  - *Positive*: Clear auditing and transparent UX for users and hackathon judges.

---

### ADR-007: Lightweight, Isolated, Lazy-Loaded Three.js Landing Hero

- **Status**: Accepted
- **Context**: The landing page benefits from an engaging 3D visualization representing position risk and collateral equilibrium. However, 3D libraries can bloat JavaScript bundle sizes, increase First Contentful Paint (FCP), and consume excessive GPU resources on mobile devices.
- **Decision**: Build the 3D hero using Three.js with strict constraints:
  - Dynamically imported on the client only (`next/dynamic` with `ssr: false`).
  - Procedural geometries only; no heavy external GLTF/GLB models or large textures.
  - Capped device pixel ratio (maximum 1.5).
  - Paused when scrolled out of view using `IntersectionObserver`.
  - Accessible fallback: respect `prefers-reduced-motion` and provide an elegant static SVG/CSS risk gauge when WebGL is unavailable.
- **Consequences**:
  - *Positive*: Distinctive visual branding without penalizing application load times or dashboard performance.
  - *Positive*: Accessible to users with motion sensitivity or low-power hardware.
  - *Negative*: Requires custom procedural scene logic rather than off-the-shelf 3D model loaders.

---

### ADR-008: Adoption of Anthropic `frontend-design` Agent Skill

- **Status**: Accepted
- **Context**: AI-generated frontend applications often suffer from generic, templated aesthetics (purple gradients, cookie-cutter SaaS cards, clunky animations). The user requested that the agent act as an opinionated design lead producing an editorial fintech aesthetic.
- **Decision**: Install and lock the official Anthropic `frontend-design` skill in `.agents/skills/frontend-design/` via `npx skills add`. Establish the visual direction:
  - Palette: Porcelain / warm off-white content surfaces, deep ink/graphite instrument panels, restrained safety amber, and deep mineral teal.
  - Typography: Editorial display typeface paired with a highly legible monospace/sans for financial metrics.
  - Information hierarchy: Prioritize scannable risk metrics (Health Factor, liquidation distance) over decorative chrome.
- **Consequences**:
  - *Positive*: Unique, professional, high-trust fintech feel appropriate for risk management software.
  - *Positive*: Consistent design tokens codified in `TOOLCHAIN.md` and `apps/web/README.md`.

---

### ADR-009: Pinned Toolchain Alignment (Agave 4.1.2, Platform Tools v1.57) and SBPF Stack Optimization

- **Status**: Accepted
- **Context**: Anchor CLI 1.2.2 defaults to SBPF architecture `v3` and platform tools `v1.57`, requiring an Agave/Solana CLI line supporting modern toolchain resolution (`4.1.2`). Additionally, complex recovery instructions with large account structs can exceed the strict 4096-byte Solana function stack frame.
- **Decision**:
  1. Pin toolchain dependencies strictly to Anchor CLI 1.2.2, Agave CLI 4.1.2, and Platform Tools v1.57 (`rustc 1.95.0`).
  2. Map `[toolchain] solana_version = "4.1.2"` in `Anchor.toml`.
  3. Box large accounts (`Box<Account<'info, T>>`) in `ExecuteRecovery` to keep `try_accounts` frame under 4096 bytes.
  4. Separate execution branches into `#[inline(never)]` helper functions to prevent stack frame exhaustion during on-chain execution.
- **Consequences**:
  - *Positive*: Reliable, deterministic Anchor compilation generating production-ready SBF bytecode (`target/deploy/lend_rescue.so`) and IDL (`target/idl/lend_rescue.json`).
  - *Positive*: Zero stack overflow risk during runtime execution on Solana test-validator or Devnet.
  - *Positive*: Strictly adheres to pinned project toolchain invariants.

---

### ADR-010: Axum REST API Architecture, Provider Abstraction, and Least-Cost Ranking Invariant

- **Status**: Accepted
- **Context**: The backend API (`apps/api`) serves as the orchestration layer between web clients, external oracles/lending markets, and the deterministic domain engines. It must maintain zero-trust boundaries, support offline development and testing, guarantee numerical safety without floating-point hazards, and enforce a least-cost recovery ranking objective.
- **Decision**:
  1. **Framework & Server**: Implemented with Axum 0.8.9 and Tokio 1.51.4 under `/api/v1` routes with CORS, body limit (2MB), request timeout (30s), and `x-request-id` tracing middleware.
  2. **Zero Financial Math Reimplementation**: API handlers strictly delegate position risk calculation to `crates/risk-engine` and candidate generation/ranking to `crates/recovery-engine`.
  3. **Provider Abstraction**: Defined `MarketProvider`, `PositionProvider`, `OracleProvider` using dyn-compatible `BoxFuture`, coupled with a reproducible `DemoProvider` providing deterministic snapshots across four health stages (Healthy, Warning, Critical, Liquidatable).
  4. **Least-Cost Ranking Objective**: Upgraded recovery optimizer and ranking to sort feasible candidates by lowest `total_cost_quote` first, breaking ties by highest post-HF, lowest slippage, and deterministic action order. Direct debt repayment ranks ahead of swap-and-repay when total capital cost is lower.
  5. **Database Persistence with In-Memory Fallback**: Provided PostgreSQL 17 migrations for `position_snapshots`, `oracle_observations`, and `recovery_recommendations` with dynamic SQLx parameterization (`query(...)` and `.bind(...)`) ensuring offline compilation, falling back to thread-safe in-memory stores when `DATABASE_URL` is unset or PostgreSQL is offline.
  6. **Numerical Precision**: Prohibit `f32`/`f64`. Large token amounts and prices are serialized as decimal strings (`"raw": "1000000000"`).
- **Consequences**:
  - *Positive*: Full testability offline in sandbox without requiring an external PostgreSQL instance.
  - *Positive*: Zero float rounding errors across the network boundary.
  - *Positive*: Directly satisfies least-cost economic optimality for borrowers.

---

### ADR-011: Anchor Architecture Hardening: InitSpace Sizing, Context Method Standardization, and Explicit Authorization/Nonce Semantics

- **Status**: Accepted
- **Context**:
  1. Anchor state accounts previously used manual `LEN` constants containing arbitrary padding buffers, causing rent inefficiency and risk of under-allocation if fields change.
  2. The `RecoveryPolicy` account overloaded a single `nonce` field for both replay protection during execution and state mutations during update/revocation, which conflated execution sequence with configuration revision.
  3. Authorization lacked a clean, idiomatic representation of owner-only execution vs delegate execution, and callers were checked imperatively rather than through structured domain predicates.
  4. Public instruction handlers in `lib.rs` mixed account validation, CPI logic, and business logic inline, violating the canonical Anchor pattern where instruction logic resides in `impl<'info> AccountsStruct<'info>`.
- **Decision**:
  1. **Canonical Account Sizing with `InitSpace`**: Derive `InitSpace` on all account structs (`MarketConfig`, `MockOracle`, `Position`, `RecoveryPolicy`, `DemoSwapPool`, `RecoveryReceipt`). Replace manual byte math with `space = 8 + Account::INIT_SPACE` and remove arbitrary padding buffers.
  2. **Option<Pubkey> Delegate Representation**: Represent the authorized delegate as `Option<Pubkey>` on-chain. When `None`, recovery execution is restricted to the position owner. When `Some(delegate)`, either the owner or the delegate is authorized to execute recovery within bounded limits.
  3. **Separation of Nonce and Revision**:
     - `execution_nonce: u64`: Initialized to 0. Consumed exclusively by successful `execute_recovery` operations, providing replay protection and unique seeds for `RecoveryReceipt` PDAs.
     - `policy_revision: u64`: Initialized to 1. Incremented on configuration updates (`update_recovery_policy`) and revocation (`revoke_recovery_policy`). Ordinary recovery execution does not increment it.
     - `schema_version: u8`: Set to constant `RECOVERY_POLICY_SCHEMA_VERSION = 2` for explicit on-chain layout versioning.
  4. **Instruction Context Method Architecture**: Refactor all instruction logic into `impl<'info> AccountsStruct<'info>` context methods with `&mut self` for state mutation and `&self` for read-only access. Anchor entrypoints in `lib.rs` remain minimal one-line dispatchers.
  5. **Stack Safety Preserved**: Maintain `Box<Account<'info, T>>` in `ExecuteRecovery` to keep the deserialization stack frame below 4096 bytes and preserve `#[inline(never)]` helper methods for individual recovery action branches.
- **Consequences**:
  - *Positive*: Exact, rent-efficient account allocation calculated at compile time by Anchor macros.
  - *Positive*: Unambiguous replay protection and audit trail: `execution_nonce` strictly counts recoveries; `policy_revision` strictly counts config mutations.
  - *Positive*: Clean separation of concerns with idiomatic Anchor context methods.
  - *Positive*: Zero regression in pure Rust crates (`crates/domain-types`, `crates/recovery-engine`) and Axum backend API.

---

### ADR-012: Pyth Core Persistent On-Chain Oracle Integration, Strict Provenance Metadata, and Zero Silent Fallback

- **Status**: Accepted
- **Context**:
  1. LendRescue requires real-world Solana market prices for risk monitoring and candidate optimization.
  2. Pyth operates two distinct models on Solana: (a) Pyth Core persistent on-chain price update accounts (`PriceUpdateV2` managed by receiver `rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ`), and (b) Pyth Pro ephemeral transaction-carried updates. LendRescue monitors on-chain positions and queries persistent accounts without requiring user transaction payloads.
  3. A critical security failure in DeFi oracles is silent fallback from live feeds to mock or stale data when a network query fails, which can obscure liquidations or quote inaccurate recovery plans.
- **Decision**:
  1. **Pyth Core Account Model**: Implement a read-only `PythOracleProvider` decoding Borsh `PriceUpdateV2` accounts as well as legacy Pyth price accounts, resolving feeds by 32-byte Pyth Feed ID.
  2. **Zero-Trust Validation Pipeline**: Every price observation is validated for: (a) non-zero positive price mantissa, (b) confidence interval bounds ($\le 200$ bps), (c) freshness ($\le 60$s old), (d) plausible timestamp (rejecting future dates), (e) feed identity matching, and (f) account program ownership.
  3. **Zero Silent Fallback**: If a Pyth observation fails, expires, or returns an error, the provider strictly propagates typed `PythError` / `ApiError`. It never silently substitutes mock or synthetic values.
  4. **Strict Provenance Metadata**: Every observation explicitly carries its source tag (`Mainnet Observed`, `Surfpool Simulated`, `Devnet Executed`, or `Synthetic / Test Fixture`), ensuring frontends and services can unambiguously differentiate real telemetry from synthetic fixtures.
  5. **Hermes Support**: Configurable server-side Bearer authentication for Hermes HTTP price updates, redacting API keys in debug and log statements.
- **Consequences**:
  - *Positive*: Production-ready oracle observation layer compatible with real Solana Pyth feeds.
  - *Positive*: Fail-closed security guarantee against stale, rogue, or out-of-bounds oracle inputs.
  - *Positive*: Full deterministic testability offline via in-memory mock transports.


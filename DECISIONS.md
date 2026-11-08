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


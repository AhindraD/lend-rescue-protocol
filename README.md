# LendRescue

> **Find the safest, cheapest way to rescue an at-risk lending position.**

LendRescue monitors DeFi lending positions on Solana, evaluates liquidation risk, calculates and ranks recovery strategies in a deterministic Rust engine, and enforces owner-defined safety policies on-chain via an Anchor smart contract.

---

## Key Features

- **Execution-Aware Recovery**: Compares debt repayment, collateral addition, and swap-and-repay options, factoring in fees, slippage, and oracle confidence intervals.
- **On-Chain Policy Enforcement**: Users configure explicit spending caps, maximum slippage, minimum post-recovery Health Factor ($HF$), and expiration on-chain.
- **Atomic Rollback Guarantee**: If any policy check or oracle freshness requirement fails, the entire transaction reverts, leaving funds and account states untouched.
- **Cryptographic Execution Receipts**: Successful recoveries consume a single-use policy nonce and record a permanent on-chain `RecoveryReceipt` PDA.
- **Non-Custodial Architecture**: LendRescue never holds, stores, or transmits user private keys.
- **Editorial Fintech Console**: Modern, accessible web application featuring an interactive 3D hero visualization with reduced-motion fallback.

---

## Architecture Overview

```mermaid
flowchart LR
    A["User / Wallet"] --> B["Next.js Web App"]
    B --> C["Axum Backend API"]
    C --> D["Pure Rust Engine"]
    C --> E["Protocol Adapters"]
    B -->|"Signed Transaction"| F["Solana Devnet (Anchor Program)"]
    F --> G["DemoSwapPool & SPL Vaults"]
    F --> H["RecoveryReceipt PDA"]
```

The system enforces clear trust boundaries:
1. **Frontend (`apps/web`)**: Client-only wallet connection, telemetry display, and transaction construction.
2. **Backend (`apps/api`)**: Orchestrates data, caches state, and calls the domain engine; advisory only (no transaction signing).
3. **Domain Engine (`crates/*`)**: Pure-Rust calculation core for Health Factor, liquidation prices, and recovery optimization using checked fixed-point arithmetic.
4. **Anchor Program (`programs/lend_rescue`)**: Authoritative on-chain policy validator and atomic executor on Solana Devnet.

---

## Data-Source & Environment Labels

Every metric, card, and screen explicitly communicates its data provenance:

- **`Mainnet Observed`**: Read-only positions and oracles read from Solana Mainnet (Kamino, Pyth).
- **`Quote Only`**: Indicative DEX quotes from third-party aggregators (e.g. Jupiter); not guaranteed fills.
- **`Surfpool Simulated`**: Local fork simulation of Mainnet account state; strictly no external broadcast.
- **`Devnet Executed`**: Live, wallet-signed transaction confirmed on Solana Devnet.
- **`Demo / Synthetic`**: Controlled synthetic market, mock oracle, and Risk Lab price shock simulator.

---

## Verified Toolchain & Prerequisites

All tools are pinned to exact versions verified on Ubuntu Linux (GLIBC 2.43):

| Component | Pinned Version | Channel / Source |
|---|---|---|
| **OS / Runtime** | Ubuntu 26.04 LTS (GLIBC $\ge 2.39$) | Linux x86_64 |
| **Node.js** | `26.10.0` | Active LTS Line (`.nvmrc`) |
| **pnpm** | `12.18.0` | `packageManager` in `package.json` |
| **Rust** | `1.99.1` | Stable channel (`rust-toolchain.toml`) |
| **Cargo** | `1.99.1` | Cargo release 2026-08-27 |
| **Anchor CLI** | `1.2.2` | AVM (`avm use 1.2.2`) |
| **Solana / Agave** | `2.1.0` | Agave CLI |
| **Platform Tools** | `1.57` | SBPF v3 |
| **Surfpool** | `1.7.0` | Local fork runner |
| **Next.js** | `16.4.5` | React 19.3.1 |
| **`@solana/web3.js`**| `1.99.0` | Legacy v1 Anchor compatibility bridge |
| **Axum** | `0.8.9` | Crates.io (`axum`) |
| **Tokio** | `1.51.4` | Async runtime (`tokio`) |
| **SQLx** | `0.9.0` | Async PostgreSQL driver (`sqlx`) |
| **Utoipa** | `6.0.0` | OpenAPI generator (`utoipa`) |
| **Agent Skill** | `frontend-design` | Anthropic skill in `.agents/skills/` |

---

## Quickstart & Setup (Phase 0)

### 1. Verify Prerequisites
```bash
node --version       # v26.10.0
pnpm --version       # 12.18.0
rustc --version      # rustc 1.99.1
cargo --version      # cargo 1.99.1
anchor --version     # anchor-cli 1.2.2
solana --version     # solana-cli 2.1.0
surfpool --version   # surfpool 1.7.0
```

### 2. Verify Manifests & Tools
```bash
# Verify Rust workspace manifest
cargo metadata --format-version 1 --no-deps

# Verify installed agent design skill
ls -la .agents/skills/frontend-design
```

---

## Documentation Index

- [`SPEC.md`](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/SPEC.md) — Comprehensive product specification, functional and non-functional requirements.
- [`LETTER_OF_INTENT.md`](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/LETTER_OF_INTENT.md) — Product vision, problem definition, and hackathon scope boundaries.
- [`ARCHITECTURE.md`](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/ARCHITECTURE.md) — Component architecture, sequence diagrams, and trust boundaries.
- [`USER_STORIES.md`](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/USER_STORIES.md) — User stories, acceptance criteria, and adversarial test scenarios.
- [`TOOLCHAIN.md`](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/TOOLCHAIN.md) — Pinned toolchain, installation commands, and compatibility guide.
- [`DECISIONS.md`](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/DECISIONS.md) — Architecture Decision Records (ADR-001 through ADR-008).
- [`SECURITY.md`](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/SECURITY.md) — Threat model, on-chain invariant rules, and vulnerability reporting.
- [`AGENTS.md`](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/AGENTS.md) — Phased implementation guidelines for AI coding agents.

---

## Development Progress

LendRescue is built following strict phased gates. Completion is tracked authoritatively in this table:

| Phase | Description | Status | Commit / Details |
|---|---|---|---|
| **0** | **Specifications & Toolchain Lock** | Complete | [`60a2f96`](https://github.com/AhindraD/lend-rescue-protocol/commit/60a2f96) — Pinned toolchain, architecture, user stories, ADRs, Anthropic frontend-design skill |
| **1** | **Modular Anchor Program** | Complete | [`c48585b`](https://github.com/AhindraD/lend-rescue-protocol/commit/c48585b) — Modular Anchor recovery program (`programs/lend_rescue`) *(Historical note: Phase 2 discovered and corrected duplicate helper definitions in `execute_recovery.rs` in commit `a126e50` that had blocked compilation. Original Phase 1 commit was not independently reproducible.)* |
| **2** | **Anchor Integration & Security Tests** | Complete | [`b02fec1`](https://github.com/AhindraD/lend-rescue-protocol/commit/b02fec1) — 39 integration tests passed, 0 failed. Surfpool 1.7.0 used as local validator. Complete atomic rollback and security defenses verified. |
| **3** | **Pure Rust Domain & Recovery Engine** | Complete | [`5d285cb`](https://github.com/AhindraD/lend-rescue-protocol/commit/5d285cb) — Three crates: `crates/domain-types`, `crates/risk-engine`, `crates/recovery-engine`. 30 workspace tests passed. Clippy (`-D warnings`) and Anchor build passed. Checked fixed-point arithmetic. |
| **4** | **Axum Backend & Engine Integration** | Complete | [`9304120`](https://github.com/AhindraD/lend-rescue-protocol/commit/9304120) — Pinned crate `lend-rescue-api` (`apps/api`). REST API `/api/v1` for markets, positions, oracles, risk, recovery quotes, optimization, and OpenAPI 3.1 JSON. Least-cost ranking objective verified. 17 API tests passed, 47 workspace tests passed, Clippy passed, Anchor 39 tests passed. |
| **5** | **Protocol Adapters (Kamino, Pyth, Jupiter, Surfpool)** | Pending | Next planned phase. |
| **6** | **Frontend Web Console & 3D Hero** | Pending | Next planned phase. |
| **7** | **End-to-End Devnet Verification** | Pending | Final demonstration phase. |

### Architectural References:
- [SPEC.md](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/SPEC.md) — Comprehensive technical requirements and domain invariants.
- [ARCHITECTURE.md](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/ARCHITECTURE.md) — Trust boundaries, data flows, and module responsibilities.
- [TOOLCHAIN.md](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/TOOLCHAIN.md) — Verified toolchain versions and pinning rules.
- [DECISIONS.md](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/DECISIONS.md) — ADRs (ADR-001 through ADR-010).
- [USER_STORIES.md](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/USER_STORIES.md) — End-to-end user stories and test expectations.
- [SECURITY.md](file:///home/kibi/web3projects/hackathons/mlh/lend-rescue-protocol/SECURITY.md) — Threat model, failure modes, and security invariants.

---

## Current Implementation

The repository currently contains:
- **Phase 1 & 2 Anchor Program (`programs/lend_rescue`)**: Deployed SBPF v3 contract with modular instructions, policy PDA validation, single-use nonce consumption, and `RecoveryReceipt` generation. Verified against 39 integration tests in Surfpool.
- **Phase 3 Deterministic Domain Engine (`crates/*`)**:
  - `crates/domain-types`: Domain models (`TokenAmount`, `Price`, `PositionSnapshot`, `RecoveryPolicy`, `HealthFactor`).
  - `crates/risk-engine`: Checked fixed-point calculations, collateral/debt valuation, synthetic Health Factor, liquidation prices, and oracle confidence interval validation.
  - `crates/recovery-engine`: Feasibility filtering, candidate plan generation, and least-cost ranking optimizer.
- **Phase 4 Axum Backend API (`apps/api`)**:
  - Package: `lend-rescue-api`.
  - Versioned REST API under `/api/v1` exposing position risk snapshots, oracle prices, recovery quotes, optimization, recommendation history, and OpenAPI JSON.
  - Decoupled `MarketProvider`, `PositionProvider`, `OracleProvider`, and deterministic `DemoProvider` (reproducible fixtures for Healthy, Warning, Critical, and Liquidatable positions).
  - PostgreSQL 17 persistence (`apps/api/migrations/`) with automatic thread-safe in-memory fallback for offline or zero-dependency runs.
  - Strict numerical safety: no floating-point primitives (`f32`, `f64`), raw token amounts serialized as decimal strings.

### Known Limitations:
- Real Mainnet Kamino accounts and live Jupiter quote APIs are stubbed through the deterministic `DemoProvider` until Phase 5 protocol adapters are implemented.
- PostgreSQL database persistence uses an in-memory fallback store when `DATABASE_URL` is unset or PostgreSQL is not running.

### Next Phase:
- **Phase 5: Protocol Adapters**: Kamino read-only obligation decoding, Pyth/Mock oracle feed ingestion, Jupiter swap quote adapter, and Surfpool local fork runner.

---

## Backend API Service (`apps/api`)

### Configuration & Environment
Create `apps/api/.env` (or copy from `apps/api/.env.example`):
```bash
BIND_HOST=127.0.0.1
BIND_PORT=3001
CORS_ALLOWED_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
DEMO_MODE=true
REQUEST_TIMEOUT_SECS=30
LOG_LEVEL=info
# Optional: PostgreSQL 17 database URL (omitting uses thread-safe in-memory store)
# DATABASE_URL=postgres://postgres:postgres@localhost:5432/lend_rescue
```

### Running the Backend
```bash
cargo run -p lend-rescue-api
```

### Running Tests
```bash
# Run API integration and unit tests (17 tests)
cargo test -p lend-rescue-api

# Run entire workspace tests (47 tests)
cargo test --workspace

# Lint with clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### REST API Endpoints (`/api/v1`)
- `GET  /api/v1/health` — System health and uptime
- `GET  /api/v1/ready` — Provider and database readiness
- `GET  /api/v1/markets` — Active lending markets
- `GET  /api/v1/positions` — Lending positions snapshot list
- `GET  /api/v1/positions/:id` — Single position snapshot
- `GET  /api/v1/positions/:id/risk` — Evaluated risk profile & synthetic Health Factor
- `GET  /api/v1/oracles/:asset_id` — Oracle price observation
- `POST /api/v1/recovery/quote` — Build single recovery plan quote
- `POST /api/v1/recovery/optimize` — Generate and rank recovery plans by least cost
- `GET  /api/v1/recovery/:id` — Fetch stored recommendation history
- `GET  /api/v1/openapi.json` — Generated OpenAPI 3.1 JSON specification

---

## Security Disclaimer

LendRescue is an experimental research and hackathon project demonstrated on Solana Devnet. It does not custody user funds. Smart contracts have not undergone a commercial security audit. Do not deploy to Solana Mainnet with real capital without an independent third-party audit.

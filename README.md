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

## Phased Implementation Roadmap

- [x] **Phase 0: Project Foundation & Toolchain Lock**
  Establish root specifications, architecture diagrams, user stories, ADRs, agent instructions, design skill, and toolchain lock.
- [ ] **Phase 1: Modular Anchor Program**
  Implement the synthetic recovery program (`programs/lend_rescue`) with modular instructions and state.
- [ ] **Phase 2: Anchor Integration Tests**
  Comprehensive happy-path and failure test suite proving atomic rollback.
- [ ] **Phase 3: Pure Rust Domain & Recovery Engine**
  Implement fixed-point math, synthetic Health Factor, liquidation prices, and candidate ranking in `crates/*`.
- [ ] **Phase 4: Backend API & Orchestration**
  Build Axum REST API and database caching layer.
- [ ] **Phase 5: Protocol Adapters**
  Integrate Kamino read-only, Pyth/Mock oracle, Jupiter quote, and Surfpool fork adapters.
- [ ] **Phase 6: Frontend Console & 3D Hero**
  Build Next.js web application with editorial design system and procedural Three.js hero.
- [ ] **Phase 7: End-to-End Devnet Verification**
  Demonstrate complete synthetic recovery and rejected out-of-policy execution on Devnet.

---

## Security Disclaimer

LendRescue is an experimental research and hackathon project demonstrated on Solana Devnet. It does not custody user funds. Smart contracts have not undergone a commercial security audit. Do not deploy to Solana Mainnet with real capital without an independent third-party audit.

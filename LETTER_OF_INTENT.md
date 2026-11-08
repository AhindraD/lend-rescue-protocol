# Letter of Intent: LendRescue

- **Product Name:** LendRescue
- **Tagline:** Find the safest, cheapest way to rescue an at-risk lending position.
- **Repository:** `lend-rescue-protocol`
- **Date:** November 2026

---

## 1. The Core Problem: Liquidation and the Execution Gap

Decentralized lending protocols are foundational to the DeFi economy. They allow capital owners to unlock liquidity by borrowing against digital assets without triggering taxable disposal events or giving up upside potential.

However, leveraged borrowing carries inherent downside risk: market price fluctuations. When the price of deposited collateral declines or accrued borrow interest accumulates, the margin of safety shrinks. Once a position crosses its liquidation threshold ($HF \le 1.0$), liquidation engines allow external actors (liquidators) to seize borrower collateral at a substantial discount—typically 5% to 15% below market price. This incurs severe, irreversible financial penalties on borrowers and, during extreme market stress, risks systemic bad debt across protocols.

Today, borrowers are not blind to liquidation risk, but they are paralyzed by the **execution gap**:
- Dashboards and notification bots inform users that a position is approaching liquidation, but they do not provide a verified, executable plan to resolve it.
- Under severe market volatility and emotional stress, working out the optimal action—whether to repay debt with liquid wallet tokens, post scarce additional collateral, or swap a portion of collateral to pay down debt—requires complex multi-step arithmetic involving oracle freshness, slippage, and protocol fees.
- Existing automation tools often require users to surrender custody of private keys or grant unrestricted transaction signing permissions to off-chain bots, creating catastrophic counterparty and smart-contract exploit risks.

---

## 2. Our Mission: Calibrated, Bounded Lending Position Recovery

LendRescue was conceived to close the gap between risk detection and safe risk remediation.

Our guiding principle is that **users should never be forced to choose between the certainty of liquidation and the risk of giving up custody**.

LendRescue provides:
1. **Mathematical Clarity**: A deterministic, open-source calculation engine that evaluates all available recovery routes (repay debt, add collateral, swap-and-repay) and transparently ranks them by capital efficiency, execution cost, and resulting safety margin.
2. **On-Chain Policy Enforcement**: Rather than relying on trusted off-chain keepers, LendRescue introduces an on-chain policy layer. Borrowers define explicit, immutable parameters—maximum capital expenditure, maximum acceptable slippage, minimum target Health Factor, expiration, and nonces.
3. **Atomic Execution Guarantees**: When a recovery plan executes, the on-chain Anchor program independently re-derives the position's state and enforces every user-configured constraint. If any parameter is violated, the transaction reverts atomically, leaving funds and account states completely untouched.

---

## 3. Target Users

LendRescue is built for participants who actively borrow in decentralized markets and value security, transparency, and self-custody:
- **DeFi Borrowers & Yield Farmers**: Capital allocators seeking automated guardrails against sudden liquidation cascades while retaining full custody of their assets.
- **Liquidity Managers & Treasury Operators**: DAOs and investment teams managing protocol debt positions that require programmatic, verifiable on-chain limits for risk mitigation.
- **Protocol Keepers & Delegates**: Actors authorized to execute defensive transactions on behalf of borrowers strictly within pre-signed, bounded policy constraints.

---

## 4. Why Solana and On-Chain Enforcement?

Solana is the ideal blockchain for LendRescue for three reasons:
- **Sub-Second Finality**: In volatile markets, price changes occur in seconds. Solana's high-speed block times allow rapid recovery execution before liquidators trigger liquidation events.
- **Low Transaction Overhead**: Executing defensive multi-step transactions (withdraw, swap, repay) remains economically practical even for smaller loan positions, whereas high network gas fees on other chains often exceed the liquidation penalty itself.
- **Composable Token Standards & Anchor Architecture**: Solana’s SPL token ecosystem and Anchor framework enable modular, verifiable CPIs (Cross-Program Invocations) with strict account validation and atomic rollback guarantees.

---

## 5. What the Hackathon MVP Proves (and What It Does Not)

### What This MVP Proves:
- A complete, working, and verifiable synthetic lending recovery lifecycle on Solana Devnet.
- Deterministic risk analysis and candidate ranking in a pure-Rust engine using checked fixed-point arithmetic.
- Independent on-chain enforcement of owner-defined recovery policies via the `lend_rescue` Anchor program.
- The generation of an immutable, verifiable `RecoveryReceipt` upon successful execution.
- Proof that unauthorized or out-of-policy recovery transactions revert cleanly, preserving 100% of account balances and states.
- Read-only observation of real Mainnet lending positions from Kamino and Pyth oracle accounts.

### What This MVP Explicitly Does Not Claim:
- It is not a new lending protocol and does not manage depositor liquidity pools.
- It is not an AMM or decentralized exchange; Devnet swaps utilize a deterministic synthetic pool (`DemoSwapPool`).
- It does not broadcast live state-altering transactions to Solana Mainnet.
- It does not operate an autonomous AI agent with discretionary signing power or fund custody.
- It makes no claims of formal legal or trademark clearance for the working product name "LendRescue".
- It makes no claims of unverified third-party protocol partnerships or audited mainnet production readiness.

LendRescue is an engineering-first, security-conscious step toward self-custodial lending defense.

import type { Metadata } from "next";
import Link from "next/link";
import { HeroSection } from "@/components/marketing/hero/hero-section";
import { RecoverySimulator } from "@/components/marketing/simulator/recovery-simulator";

export const metadata: Metadata = {
  title: "LendRescue — Lending Risk Intelligence & Recovery",
  description:
    "Understand lending risk. Compare recovery options. Make a better-informed decision before liquidation cascades strike.",
};

export default function MarketingPage() {
  return (
    <div className="flex flex-col min-h-screen">
      {/* 1. Cinematic Hero with Three.js Rescue Core */}
      <HeroSection />

      {/* 2. The Problem: The DeFi Liquidation Trap */}
      <section
        id="problem"
        className="py-24 px-6 border-t border-[var(--border)] bg-[var(--surface-1)] relative"
      >
        <div className="mx-auto max-w-7xl">
          <div className="text-center max-w-2xl mx-auto mb-16">
            <span className="text-xs font-mono uppercase tracking-wider text-[var(--risk-warning)] block mb-2">
              The Liquidation Trap
            </span>
            <h2 className="text-3xl sm:text-4xl font-extrabold text-[var(--text-primary)] tracking-tight">
              Why Solana Lending Positions Fail Abruptly
            </h2>
            <p className="text-[var(--text-secondary)] text-sm sm:text-base mt-4">
              DeFi lending protocols enforce binary liquidations. When price volatility hits,
              borrowers face sudden penalties without actionable recovery insights.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="p-6 rounded-2xl border border-[var(--border)] bg-[var(--surface-2)]/60 space-y-4">
              <div className="h-10 w-10 rounded-xl bg-[var(--risk-critical)]/10 border border-[var(--risk-critical)]/30 flex items-center justify-center">
                <span className="text-[var(--risk-critical)] font-mono font-bold">01</span>
              </div>
              <h3 className="text-lg font-bold text-[var(--text-primary)]">
                Sudden Oracle Cascades
              </h3>
              <p className="text-sm text-[var(--text-secondary)] leading-relaxed">
                Market downturns can erode collateral value within minutes. Traditional frontends
                update infrequently, obscuring real-time Pyth price confidence intervals.
              </p>
            </div>

            <div className="p-6 rounded-2xl border border-[var(--border)] bg-[var(--surface-2)]/60 space-y-4">
              <div className="h-10 w-10 rounded-xl bg-[var(--risk-warning)]/10 border border-[var(--risk-warning)]/30 flex items-center justify-center">
                <span className="text-[var(--risk-warning)] font-mono font-bold">02</span>
              </div>
              <h3 className="text-lg font-bold text-[var(--text-primary)]">
                Punitive Haircuts
              </h3>
              <p className="text-sm text-[var(--text-secondary)] leading-relaxed">
                Liquidation bots claim up to 5%–10% liquidation penalties on your collateral,
                often wiping out entire positions during rapid network volatility.
              </p>
            </div>

            <div className="p-6 rounded-2xl border border-[var(--border)] bg-[var(--surface-2)]/60 space-y-4">
              <div className="h-10 w-10 rounded-xl bg-[var(--accent)]/10 border border-[var(--accent)]/30 flex items-center justify-center">
                <span className="text-[var(--accent)] font-mono font-bold">03</span>
              </div>
              <h3 className="text-lg font-bold text-[var(--text-primary)]">
                No Plan Comparison
              </h3>
              <p className="text-sm text-[var(--text-secondary)] leading-relaxed">
                Borrowers are left guessing whether to deposit more SOL, swap collateral via Jupiter,
                or partially repay debt, without quantitative comparisons of health impact.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* 3. Interactive Strategy Simulator */}
      <section
        id="simulator"
        className="py-24 px-6 border-t border-[var(--border)] bg-[var(--background)]"
      >
        <div className="mx-auto max-w-7xl">
          <RecoverySimulator />
        </div>
      </section>

      {/* 4. How It Works: The Deterministic Engine */}
      <section
        id="how-it-works"
        className="py-24 px-6 border-t border-[var(--border)] bg-[var(--surface-1)]"
      >
        <div className="mx-auto max-w-7xl">
          <div className="text-center max-w-2xl mx-auto mb-16">
            <span className="text-xs font-mono uppercase tracking-wider text-[var(--accent)] block mb-2">
              Architecture
            </span>
            <h2 className="text-3xl sm:text-4xl font-extrabold text-[var(--text-primary)] tracking-tight">
              Deterministic Lending Intelligence
            </h2>
            <p className="text-[var(--text-secondary)] text-sm sm:text-base mt-4">
              LendRescue integrates directly with on-chain protocol state and off-chain calculation engines.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            {[
              {
                step: "01",
                badge: "State Ingestion",
                title: "Kamino SDK & Reserves",
                desc: "Queries real-time obligations, collateral deposits, loan-to-value limits, and borrow reserves from Kamino Lending Main Market.",
              },
              {
                step: "02",
                badge: "Push Oracles",
                title: "Pyth Core Price Feeds",
                desc: "Ingests sub-second Pyth confidence intervals and publish timestamps to verify freshness and eliminate stale price vulnerabilities.",
              },
              {
                step: "03",
                badge: "Rust Optimizer",
                title: "Deterministic Engine",
                desc: "Calculates exact health factors, liquidation prices, and generates ranked recovery candidate combinations via the Axum backend.",
              },
              {
                step: "04",
                badge: "Indicative Routing",
                title: "Jupiter Swap V2",
                desc: "Fetches live route plans, estimated slippage, and output tokens for swap-assisted rebalancing—strictly quote-only.",
              },
            ].map((card) => (
              <div
                key={card.step}
                className="p-6 rounded-2xl border border-[var(--border)] bg-[var(--surface-2)]/40 flex flex-col justify-between"
              >
                <div>
                  <div className="flex items-center justify-between mb-4">
                    <span className="text-xs font-mono font-bold text-[var(--accent)]">
                      {card.step}
                    </span>
                    <span className="text-[10px] font-mono px-2 py-0.5 rounded border border-[var(--border)] bg-[var(--surface-3)] text-[var(--text-muted)]">
                      {card.badge}
                    </span>
                  </div>
                  <h3 className="text-base font-bold text-[var(--text-primary)] mb-2">
                    {card.title}
                  </h3>
                  <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
                    {card.desc}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* 5. Protocol Integrations */}
      <section
        id="protocols"
        className="py-24 px-6 border-t border-[var(--border)] bg-[var(--background)]"
      >
        <div className="mx-auto max-w-7xl">
          <div className="text-center max-w-2xl mx-auto mb-16">
            <span className="text-xs font-mono uppercase tracking-wider text-[var(--risk-safe)] block mb-2">
              Ecosystem
            </span>
            <h2 className="text-3xl sm:text-4xl font-extrabold text-[var(--text-primary)] tracking-tight">
              Built on Battle-Tested Solana Infrastructure
            </h2>
            <p className="text-[var(--text-secondary)] text-sm sm:text-base mt-4">
              Every adapter is strictly read-only and designed for zero-trust operation.
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
            {[
              {
                name: "Kamino Lending",
                type: "Lending Market Protocol",
                desc: "Decodes Main Market reserves and user obligations into normalized domain types.",
                badge: "Read-Only SDK",
                accent: "text-[var(--accent)]",
              },
              {
                name: "Pyth Network Core",
                type: "Push Oracle Feeds",
                desc: "High-frequency SOL/USD and USDC/USD price observations with confidence metrics.",
                badge: "Verified Oracles",
                accent: "text-[var(--risk-safe)]",
              },
              {
                name: "Jupiter Swap V2",
                type: "DEX Aggregator",
                desc: "Precise multi-hop quote evaluation, price impact checks, and route optimization.",
                badge: "Quote-Only",
                accent: "text-[var(--risk-warning)]",
              },
              {
                name: "Solana Anchor SVM",
                type: "On-Chain Policy Engine",
                desc: "Modular Anchor program enforcing recovery rules and non-custodial safety checks.",
                badge: "SBPF v3",
                accent: "text-[var(--text-primary)]",
              },
              {
                name: "Surfpool Fork",
                type: "Mainnet Simulation",
                desc: "Local fork harness validating adapter responses against actual mainnet account data.",
                badge: "Simulated Testing",
                accent: "text-[var(--text-secondary)]",
              },
              {
                name: "Axum Rust Backend",
                type: "High-Performance API",
                desc: "Microsecond risk scoring, SQLite history caching, and OpenAPI specification.",
                badge: "Deterministic API",
                accent: "text-[var(--accent)]",
              },
            ].map((p) => (
              <div
                key={p.name}
                className="p-6 rounded-2xl border border-[var(--border)] bg-[var(--surface-1)] hover:border-[var(--border)]/80 transition-colors"
              >
                <div className="flex items-center justify-between mb-3">
                  <span className="text-xs font-mono uppercase text-[var(--text-muted)]">
                    {p.type}
                  </span>
                  <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-[var(--surface-2)] text-[var(--text-muted)] border border-[var(--border)]">
                    {p.badge}
                  </span>
                </div>
                <h3 className={`text-base font-bold mb-1.5 ${p.accent}`}>
                  {p.name}
                </h3>
                <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
                  {p.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* 6. Security Guarantees */}
      <section
        id="security"
        className="py-24 px-6 border-t border-[var(--border)] bg-[var(--surface-1)]"
      >
        <div className="mx-auto max-w-5xl">
          <div className="text-center max-w-2xl mx-auto mb-16">
            <span className="text-xs font-mono uppercase tracking-wider text-[var(--risk-safe)] block mb-2">
              Safety Guarantees
            </span>
            <h2 className="text-3xl sm:text-4xl font-extrabold text-[var(--text-primary)] tracking-tight">
              Non-Custodial by Architecture
            </h2>
            <p className="text-[var(--text-secondary)] text-sm sm:text-base mt-4">
              LendRescue is fundamentally an advisory intelligence tool. It never touches user funds.
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {[
              {
                title: "No Private Key or Wallet Signing",
                desc: "The frontend does not request, hold, or store private keys. Risk monitoring operates strictly using public wallet addresses.",
              },
              {
                title: "No Automated Transaction Broadcasts",
                desc: "LendRescue cannot initiate or broadcast transactions on behalf of users. Recovery plans are purely advisory recommendations.",
              },
              {
                title: "Server-Side Mathematical Precision",
                desc: "Zero financial calculations are trusted to browser JavaScript. All health factors, liquidation thresholds, and candidate rankings come from verified Rust engines.",
              },
              {
                title: "Multi-Asset Fail-Closed Safety",
                desc: "Complex positions with multi-collateral or multi-debt exposures return explicit HTTP 422 errors instead of silent incorrect approximations.",
              },
              {
                title: "Indicative Jupiter Disclaimers",
                desc: "DEX quotes are explicitly marked as indicative snapshots with no automated swap execution or slippage risk.",
              },
              {
                title: "Verifiable Data Provenance",
                desc: "Every oracle and market reading carries a verified provenance badge (Live, Simulated, Cached, or Mock) with timestamp staleness indicators.",
              },
            ].map((item) => (
              <div
                key={item.title}
                className="p-5 rounded-xl border border-[var(--border)] bg-[var(--surface-2)]/50 flex items-start gap-4"
              >
                <div className="h-6 w-6 rounded-full bg-[var(--risk-safe)]/10 border border-[var(--risk-safe)]/30 flex items-center justify-center shrink-0 mt-0.5">
                  <span className="text-[var(--risk-safe)] text-xs font-bold">✓</span>
                </div>
                <div>
                  <h4 className="text-sm font-bold text-[var(--text-primary)] mb-1">
                    {item.title}
                  </h4>
                  <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
                    {item.desc}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* 7. Bottom CTA */}
      <section className="py-28 px-6 border-t border-[var(--border)] bg-[var(--background)] relative overflow-hidden">
        <div className="mx-auto max-w-4xl text-center space-y-6 relative z-10">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-[var(--accent)]/30 bg-[var(--accent)]/5 text-xs font-mono text-[var(--accent)]">
            <span className="h-1.5 w-1.5 rounded-full bg-[var(--accent)] animate-pulse" />
            <span>Ready for Production Inspection</span>
          </div>

          <h2 className="text-4xl sm:text-5xl font-extrabold text-[var(--text-primary)] tracking-tight">
            Take Control of Your Lending Risk Today
          </h2>

          <p className="text-base sm:text-lg text-[var(--text-secondary)] max-w-2xl mx-auto leading-relaxed">
            Monitor active obligations, analyze Pyth oracle feeds, and explore ranked recovery
            options across Kamino Lending with zero custody risk.
          </p>

          <div className="pt-4 flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link
              href="/app/dashboard"
              className="w-full sm:w-auto px-9 py-4 rounded-lg bg-[var(--accent)] text-[var(--background)] font-bold text-base hover:bg-[var(--accent)]/90 transition-all shadow-xl shadow-[var(--accent)]/15"
            >
              Launch Console Now →
            </Link>
          </div>
        </div>

        {/* Ambient background glow */}
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] bg-[var(--accent)]/5 rounded-full blur-3xl pointer-events-none -z-10" />
      </section>
    </div>
  );
}

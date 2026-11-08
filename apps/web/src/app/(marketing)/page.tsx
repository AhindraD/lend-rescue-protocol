import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = {
  title: "LendRescue — Lending Risk Intelligence & Recovery",
  description:
    "Understand lending risk. Compare recovery options. Make a better-informed decision before liquidation risk becomes critical.",
};

export default function MarketingPage() {
  return (
    <div>
      {/* Hero Section */}
      <section className="relative overflow-hidden py-28 md:py-36 px-6">
        <div className="mx-auto max-w-4xl text-center">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-[var(--accent)]/30 bg-[var(--accent)]/5 text-[var(--accent)] text-xs font-mono mb-8">
            <span className="h-1.5 w-1.5 rounded-full bg-[var(--accent)] animate-pulse" />
            Solana · Non-Custodial · Advisory Intelligence
          </div>
          <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold tracking-tight text-[var(--text-primary)] mb-6 leading-tight">
            Understand lending risk.{" "}
            <span className="text-[var(--accent)] block sm:inline">
              Avoid liquidation.
            </span>
          </h1>
          <p className="text-lg sm:text-xl text-[var(--text-secondary)] max-w-2xl mx-auto mb-10 leading-relaxed">
            LendRescue monitors your Solana lending positions across Kamino,
            analyzes real-time Pyth oracle health factors, and models ranked recovery
            strategies with indicative Jupiter swap quotes.
          </p>
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link
              href="/app/dashboard"
              className="w-full sm:w-auto px-8 py-3.5 rounded-lg bg-[var(--accent)] text-[var(--background)] font-semibold text-base hover:bg-[var(--accent)]/90 transition-colors text-center shadow-lg shadow-[var(--accent)]/10"
            >
              Open Console →
            </Link>
            <a
              href="#how-it-works"
              className="w-full sm:w-auto px-8 py-3.5 rounded-lg border border-[var(--border)] text-[var(--text-secondary)] text-base hover:text-[var(--text-primary)] hover:border-[var(--text-muted)] transition-colors text-center bg-[var(--surface-1)]"
            >
              How It Works
            </a>
          </div>
        </div>

        {/* Ambient grid background */}
        <div
          className="absolute inset-0 -z-10 opacity-20 pointer-events-none"
          style={{
            backgroundImage:
              "linear-gradient(var(--border) 1px, transparent 1px), linear-gradient(90deg, var(--border) 1px, transparent 1px)",
            backgroundSize: "60px 60px",
          }}
        />
      </section>

      {/* How It Works */}
      <section
        id="how-it-works"
        className="py-24 px-6 border-t border-[var(--border)] bg-[var(--surface-1)]"
      >
        <div className="mx-auto max-w-6xl">
          <div className="text-center max-w-xl mx-auto mb-16">
            <span className="text-xs font-mono uppercase tracking-wider text-[var(--accent)] mb-2 block">
              Workflow
            </span>
            <h2 className="text-3xl font-bold text-[var(--text-primary)] mb-4">
              How LendRescue Works
            </h2>
            <p className="text-[var(--text-secondary)] text-sm">
              Three transparent steps from position awareness to informed recovery strategy.
            </p>
          </div>
          <div className="grid md:grid-cols-3 gap-8">
            {[
              {
                step: "01",
                title: "Inspect Lending Positions",
                description:
                  "Query any Solana wallet or Kamino obligation address. LendRescue decodes on-chain reserves, deposits, debts, and loan-to-value limits via official protocol state.",
              },
              {
                step: "02",
                title: "Calculate Risk Metrics",
                description:
                  "Our deterministic risk engine computes exact health factors, liquidation thresholds, and time-to-critical estimates using verified Pyth Core oracle prices.",
              },
              {
                step: "03",
                title: "Compare Recovery Plans",
                description:
                  "Model ranked recovery candidates—debt repayments, collateral swaps, or rebalancing. Jupiter API V2 provides indicative quotes with zero execution risk.",
              },
            ].map((item) => (
              <div
                key={item.step}
                className="rounded-xl border border-[var(--border)] bg-[var(--surface-2)] p-6 flex flex-col justify-between"
              >
                <div>
                  <span className="text-[var(--accent)] font-mono text-sm font-bold">
                    {item.step}
                  </span>
                  <h3 className="mt-3 text-base font-semibold text-[var(--text-primary)] mb-2">
                    {item.title}
                  </h3>
                  <p className="text-sm text-[var(--text-secondary)] leading-relaxed">
                    {item.description}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Protocol Integrations */}
      <section
        id="protocols"
        className="py-24 px-6 border-t border-[var(--border)] bg-[var(--background)]"
      >
        <div className="mx-auto max-w-6xl">
          <div className="text-center max-w-xl mx-auto mb-16">
            <span className="text-xs font-mono uppercase tracking-wider text-[var(--accent)] mb-2 block">
              Ecosystem
            </span>
            <h2 className="text-3xl font-bold text-[var(--text-primary)] mb-4">
              Protocol Integrations
            </h2>
            <p className="text-[var(--text-secondary)] text-sm">
              Read-only data from industry-standard Solana protocols with zero private-key exposure.
            </p>
          </div>
          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
            {[
              {
                name: "Kamino Lending",
                role: "Lending markets, reserves & obligations",
                badge: "Read-only",
              },
              {
                name: "Pyth Core Network",
                role: "Confidence-aware price oracles",
                badge: "Live Feed",
              },
              {
                name: "Jupiter Swap V2",
                role: "Indicative swap routing & price impact",
                badge: "Quote-only",
              },
              {
                name: "Solana Anchor SVM",
                role: "Verifiable policy contracts",
                badge: "Non-custodial",
              },
              {
                name: "Surfpool Fork",
                role: "Local mainnet account simulation",
                badge: "Simulated",
              },
              {
                name: "Axum Risk Engine",
                role: "High-performance Rust calculation backend",
                badge: "Deterministic",
              },
            ].map((p) => (
              <div
                key={p.name}
                className="rounded-xl border border-[var(--border)] bg-[var(--surface-1)] p-5 flex items-start gap-4"
              >
                <div className="h-10 w-10 rounded-lg bg-[var(--surface-2)] border border-[var(--border)] flex items-center justify-center shrink-0">
                  <span className="text-[var(--accent)] font-mono text-sm font-bold">
                    ◈
                  </span>
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-semibold text-[var(--text-primary)]">
                    {p.name}
                  </p>
                  <p className="text-xs text-[var(--text-secondary)] mt-0.5">
                    {p.role}
                  </p>
                  <span className="inline-block mt-2 px-2 py-0.5 rounded text-xs font-mono bg-[var(--surface-3)] text-[var(--text-muted)] border border-[var(--border)]">
                    {p.badge}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Security Architecture */}
      <section
        id="security"
        className="py-24 px-6 border-t border-[var(--border)] bg-[var(--surface-1)]"
      >
        <div className="mx-auto max-w-4xl text-center">
          <span className="text-xs font-mono uppercase tracking-wider text-[var(--risk-safe)] mb-2 block">
            Verification
          </span>
          <h2 className="text-3xl font-bold text-[var(--text-primary)] mb-4">
            Security &amp; Non-Custodial Boundaries
          </h2>
          <p className="text-[var(--text-secondary)] text-sm max-w-2xl mx-auto mb-12">
            LendRescue was built with strict safety invariants. It is an advisory intelligence
            console that never holds user assets, constructs unsigned transactions, or broadcasts to mainnet.
          </p>
          <div className="grid sm:grid-cols-2 gap-4 text-left">
            {[
              "No private keys or wallet seeds requested or stored",
              "No transaction broadcasting or signing capabilities",
              "Zero browser-side approximations of financial health factors",
              "Multi-asset Kamino obligations fail-closed with explicit error reporting",
              "Indicative Jupiter quotes are explicitly labeled and not executed",
              "Every data observation carries provenance and timestamp indicators",
            ].map((item) => (
              <div
                key={item}
                className="flex items-start gap-3 rounded-xl border border-[var(--border)] bg-[var(--surface-2)] p-4"
              >
                <svg
                  className="h-5 w-5 text-[var(--risk-safe)] mt-0.5 shrink-0"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                  aria-hidden="true"
                >
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z"
                    clipRule="evenodd"
                  />
                </svg>
                <span className="text-sm text-[var(--text-secondary)] leading-snug">
                  {item}
                </span>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-24 px-6 border-t border-[var(--border)] bg-[var(--background)]">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-bold text-[var(--text-primary)] mb-4">
            Inspect Position Risk Now
          </h2>
          <p className="text-[var(--text-secondary)] text-sm mb-8">
            Access the lending console to monitor obligations and compare recovery quotes.
          </p>
          <Link
            href="/app/dashboard"
            className="inline-flex items-center px-8 py-3.5 rounded-lg bg-[var(--accent)] text-[var(--background)] font-semibold text-base hover:bg-[var(--accent)]/90 transition-colors shadow-lg shadow-[var(--accent)]/10"
          >
            Launch Console →
          </Link>
        </div>
      </section>
    </div>
  );
}

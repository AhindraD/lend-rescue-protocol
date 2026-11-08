"use client";

import dynamic from "next/dynamic";
import Link from "next/link";

const DynamicHeroScene = dynamic(
  () => import("./hero-scene").then((mod) => mod.HeroScene),
  {
    ssr: false,
    loading: () => (
      <div className="w-full h-full min-h-[420px] md:min-h-[520px] flex items-center justify-center">
        <div className="relative w-56 h-56 flex items-center justify-center">
          <div className="absolute inset-0 rounded-full border border-[var(--border)] animate-pulse" />
          <div className="absolute inset-6 rounded-full border border-[var(--accent)]/20 animate-spin" />
          <div className="h-16 w-16 rounded-full bg-[var(--surface-2)] border border-[var(--border)] flex items-center justify-center">
            <span className="text-[var(--accent)] font-mono text-2xl font-bold">◈</span>
          </div>
        </div>
      </div>
    ),
  }
);

export function HeroSection() {
  return (
    <section className="relative overflow-hidden pt-12 pb-24 md:pt-20 md:pb-32 px-6">
      <div className="mx-auto max-w-7xl">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-center">
          {/* Left Column: Headlines & Actions */}
          <div className="lg:col-span-7 space-y-6 text-center lg:text-left z-10">
            {/* Pill Badge */}
            <div className="inline-flex items-center gap-2.5 px-3 py-1.5 rounded-full border border-[var(--accent)]/30 bg-[var(--accent)]/5 text-xs font-mono text-[var(--accent)]">
              <span className="h-1.5 w-1.5 rounded-full bg-[var(--accent)] animate-pulse" />
              <span>Solana SVM · Non-Custodial · Advisory Risk Engine</span>
            </div>

            {/* Main Headline */}
            <h1 className="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight text-[var(--text-primary)] leading-[1.1]">
              Anticipate lending risk.{" "}
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-[var(--accent)] via-[#8deee0] to-[var(--risk-safe)]">
                Prevent liquidation.
              </span>
            </h1>

            {/* Subtitle */}
            <p className="text-lg sm:text-xl text-[var(--text-secondary)] max-w-2xl mx-auto lg:mx-0 leading-relaxed">
              LendRescue monitors Kamino obligations in real time, computes deterministic
              health factors with Pyth Core oracles, and compares ranked recovery
              plans with indicative Jupiter swap routing before liquidation cascades strike.
            </p>

            {/* CTA Group */}
            <div className="flex flex-col sm:flex-row items-center justify-center lg:justify-start gap-4 pt-2">
              <Link
                href="/app/dashboard"
                className="w-full sm:w-auto px-8 py-3.5 rounded-lg bg-[var(--accent)] text-[var(--background)] font-semibold text-base hover:bg-[var(--accent)]/90 transition-all shadow-lg shadow-[var(--accent)]/15 text-center flex items-center justify-center gap-2"
              >
                <span>Launch Console</span>
                <span className="text-sm font-mono">→</span>
              </Link>
              <a
                href="#simulator"
                className="w-full sm:w-auto px-7 py-3.5 rounded-lg border border-[var(--border)] text-[var(--text-secondary)] text-base hover:text-[var(--text-primary)] hover:border-[var(--text-muted)] transition-colors text-center bg-[var(--surface-1)]/60 flex items-center justify-center gap-2"
              >
                <span>Explore Simulator</span>
                <span className="text-xs font-mono text-[var(--text-muted)]">↓</span>
              </a>
            </div>

            {/* Trust Points */}
            <div className="pt-6 grid grid-cols-3 gap-4 border-t border-[var(--border)]/60 max-w-lg mx-auto lg:mx-0">
              <div>
                <p className="text-xs font-mono uppercase text-[var(--text-muted)]">Kamino Market</p>
                <p className="text-sm font-semibold text-[var(--text-primary)] font-mono mt-0.5">Main Pool</p>
              </div>
              <div>
                <p className="text-xs font-mono uppercase text-[var(--text-muted)]">Oracle Speed</p>
                <p className="text-sm font-semibold text-[var(--text-primary)] font-mono mt-0.5">Sub-second</p>
              </div>
              <div>
                <p className="text-xs font-mono uppercase text-[var(--text-muted)]">Execution Risk</p>
                <p className="text-sm font-semibold text-[var(--risk-safe)] font-mono mt-0.5">Zero (Advisory)</p>
              </div>
            </div>
          </div>

          {/* Right Column: Three.js Rescue Core Visual */}
          <div className="lg:col-span-5 relative flex items-center justify-center">
            {/* Ambient Radial Gradient Glow */}
            <div className="absolute w-72 h-72 md:w-96 md:h-96 rounded-full bg-[var(--accent)]/10 blur-3xl pointer-events-none -z-10" />

            {/* Canvas Container */}
            <div className="w-full max-w-md lg:max-w-none aspect-square relative">
              <DynamicHeroScene />
            </div>
          </div>
        </div>
      </div>

      {/* Decorative Grid Lines */}
      <div
        className="absolute inset-0 -z-20 opacity-15 pointer-events-none"
        style={{
          backgroundImage:
            "linear-gradient(var(--border) 1px, transparent 1px), linear-gradient(90deg, var(--border) 1px, transparent 1px)",
          backgroundSize: "64px 64px",
        }}
      />
    </section>
  );
}


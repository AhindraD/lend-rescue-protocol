"use client";

import { useState } from "react";
import { Badge } from "@/components/ui/badge";

interface PresetScenario {
  id: string;
  name: string;
  collateralAsset: string;
  collateralAmount: number;
  collateralPriceUsd: number;
  borrowAsset: string;
  borrowAmount: number;
  borrowPriceUsd: number;
  liquidationThresholdPct: number;
}

const SCENARIOS: PresetScenario[] = [
  {
    id: "sol-usdc",
    name: "SOL / USDC (High Volatility)",
    collateralAsset: "SOL",
    collateralAmount: 20,
    collateralPriceUsd: 150,
    borrowAsset: "USDC",
    borrowAmount: 2200,
    borrowPriceUsd: 1.0,
    liquidationThresholdPct: 80,
  },
  {
    id: "msol-usdc",
    name: "mSOL / USDC (Liquid Staking)",
    collateralAsset: "mSOL",
    collateralAmount: 18,
    collateralPriceUsd: 175,
    borrowAsset: "USDC",
    borrowAmount: 2350,
    borrowPriceUsd: 1.0,
    liquidationThresholdPct: 85,
  },
  {
    id: "sol-usdt",
    name: "SOL / USDT (Stable Debt)",
    collateralAsset: "SOL",
    collateralAmount: 35,
    collateralPriceUsd: 150,
    borrowAsset: "USDT",
    borrowAmount: 3900,
    borrowPriceUsd: 1.0,
    liquidationThresholdPct: 80,
  },
];

export function RecoverySimulator() {
  const [selectedScenario, setSelectedScenario] = useState<PresetScenario>(
    SCENARIOS[0]
  );
  const [priceShockPct, setPriceShockPct] = useState<number>(-12); // -12% shock

  // Compute illustrative values
  const currentPrice =
    selectedScenario.collateralPriceUsd * (1 + priceShockPct / 100);
  const totalCollateralValue =
    selectedScenario.collateralAmount * currentPrice;
  const totalDebtValue =
    selectedScenario.borrowAmount * selectedScenario.borrowPriceUsd;

  // Max borrow allowed before liquidation
  const liquidationThresholdValue =
    totalCollateralValue * (selectedScenario.liquidationThresholdPct / 100);

  // Health factor: liquidationThresholdValue / totalDebtValue
  const healthFactor =
    totalDebtValue > 0
      ? liquidationThresholdValue / totalDebtValue
      : 999;

  // Liquidation Price: Debt / (Collateral * LT)
  const liquidationPrice =
    totalDebtValue /
    (selectedScenario.collateralAmount *
      (selectedScenario.liquidationThresholdPct / 100));

  // Determine risk category
  let riskCategory: "Safe" | "Warning" | "Critical" = "Safe";
  let riskBadgeVariant: "safe" | "warning" | "critical" = "safe";
  let hfColor = "text-[var(--risk-safe)]";

  if (healthFactor < 1.1) {
    riskCategory = "Critical";
    riskBadgeVariant = "critical";
    hfColor = "text-[var(--risk-critical)]";
  } else if (healthFactor < 1.25) {
    riskCategory = "Warning";
    riskBadgeVariant = "warning";
    hfColor = "text-[var(--risk-warning)]";
  }

  // Simulated candidate plans
  const repayDebtAmount = Math.round(totalDebtValue * 0.25);
  const hfAfterRepay = (
    liquidationThresholdValue / (totalDebtValue - repayDebtAmount)
  ).toFixed(2);

  const addCollateralUnits = (
    selectedScenario.collateralAmount * 0.25
  ).toFixed(1);
  const hfAfterAdd = (
    ((selectedScenario.collateralAmount + parseFloat(addCollateralUnits)) *
      currentPrice *
      (selectedScenario.liquidationThresholdPct / 100)) /
    totalDebtValue
  ).toFixed(2);

  return (
    <div className="rounded-2xl border border-[var(--border)] bg-[var(--surface-1)] p-6 md:p-8 shadow-xl">
      {/* Header & Scenario Selector */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-6 border-b border-[var(--border)]">
        <div>
          <span className="text-xs font-mono uppercase tracking-wider text-[var(--accent)] block mb-1">
            Interactive Model
          </span>
          <h3 className="text-xl font-bold text-[var(--text-primary)]">
            Lending Risk &amp; Recovery Simulator
          </h3>
          <p className="text-xs text-[var(--text-secondary)] mt-1">
            Adjust collateral market shocks to simulate liquidation thresholds and recovery candidates.
          </p>
        </div>

        {/* Preset Selector */}
        <div className="flex flex-wrap gap-2">
          {SCENARIOS.map((s) => (
            <button
              key={s.id}
              onClick={() => setSelectedScenario(s)}
              className={`px-3 py-1.5 rounded-lg text-xs font-mono transition-colors cursor-pointer ${
                selectedScenario.id === s.id
                  ? "bg-[var(--accent)]/15 text-[var(--accent)] border border-[var(--accent)]/40"
                  : "bg-[var(--surface-2)] text-[var(--text-muted)] border border-[var(--border)] hover:text-[var(--text-secondary)]"
              }`}
            >
              {s.name}
            </button>
          ))}
        </div>
      </div>

      {/* Main Simulator Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 pt-6">
        {/* Left: Interactive Controls */}
        <div className="lg:col-span-5 space-y-6">
          {/* Price Shock Slider */}
          <div className="space-y-3 p-4 rounded-xl border border-[var(--border)] bg-[var(--surface-2)]/40">
            <div className="flex items-center justify-between">
              <label className="text-xs font-mono uppercase text-[var(--text-secondary)]">
                Collateral Price Shock ({selectedScenario.collateralAsset})
              </label>
              <span
                className={`text-sm font-mono font-bold ${
                  priceShockPct < 0
                    ? "text-[var(--risk-critical)]"
                    : "text-[var(--risk-safe)]"
                }`}
              >
                {priceShockPct > 0 ? `+${priceShockPct}%` : `${priceShockPct}%`}
              </span>
            </div>
            <input
              type="range"
              min="-35"
              max="15"
              step="1"
              value={priceShockPct}
              onChange={(e) => setPriceShockPct(parseInt(e.target.value, 10))}
              className="w-full accent-[var(--accent)] cursor-pointer"
            />
            <div className="flex justify-between text-[11px] font-mono text-[var(--text-muted)]">
              <span>-35% (Crash)</span>
              <span>0% (Baseline)</span>
              <span>+15% (Rally)</span>
            </div>
          </div>

          {/* Position Breakdown Table */}
          <div className="space-y-3 text-xs font-mono">
            <div className="flex justify-between py-2 border-b border-[var(--border)]/60">
              <span className="text-[var(--text-muted)]">Collateral Asset</span>
              <span className="text-[var(--text-primary)]">
                {selectedScenario.collateralAmount} {selectedScenario.collateralAsset} @ ${currentPrice.toFixed(2)}
              </span>
            </div>
            <div className="flex justify-between py-2 border-b border-[var(--border)]/60">
              <span className="text-[var(--text-muted)]">Collateral Value</span>
              <span className="text-[var(--text-primary)]">
                ${totalCollateralValue.toLocaleString(undefined, { maximumFractionDigits: 2 })}
              </span>
            </div>
            <div className="flex justify-between py-2 border-b border-[var(--border)]/60">
              <span className="text-[var(--text-muted)]">Debt Obligation</span>
              <span className="text-[var(--text-primary)]">
                {selectedScenario.borrowAmount} {selectedScenario.borrowAsset} (${totalDebtValue.toLocaleString()})
              </span>
            </div>
            <div className="flex justify-between py-2 border-b border-[var(--border)]/60">
              <span className="text-[var(--text-muted)]">Liquidation Price</span>
              <span className="text-[var(--risk-warning)]">
                ${liquidationPrice.toFixed(2)} {selectedScenario.collateralAsset}
              </span>
            </div>
          </div>
        </div>

        {/* Right: Risk Assessment & Ranked Recovery Candidates */}
        <div className="lg:col-span-7 space-y-6">
          {/* Health Factor Gauge Card */}
          <div className="p-5 rounded-xl border border-[var(--border)] bg-[var(--surface-2)]/60 flex flex-col sm:flex-row items-center justify-between gap-4">
            <div>
              <p className="text-xs font-mono uppercase text-[var(--text-muted)] mb-1">
                Estimated Health Factor
              </p>
              <div className="flex items-baseline gap-3">
                <span className={`text-4xl font-extrabold font-mono ${hfColor}`}>
                  {healthFactor.toFixed(3)}
                </span>
                <Badge variant={riskBadgeVariant}>{riskCategory}</Badge>
              </div>
              <p className="text-xs text-[var(--text-secondary)] mt-1">
                Liquidation occurs if Health Factor drops below 1.000
              </p>
            </div>

            <div className="text-right sm:text-right w-full sm:w-auto p-3 rounded-lg bg-[var(--surface-1)] border border-[var(--border)]">
              <p className="text-[11px] font-mono uppercase text-[var(--text-muted)]">Buffer to Liquidation</p>
              <p className={`text-base font-mono font-bold mt-0.5 ${hfColor}`}>
                {(((currentPrice - liquidationPrice) / currentPrice) * 100).toFixed(1)}% price drop
              </p>
            </div>
          </div>

          {/* Recovery Strategy Candidates Preview */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-xs font-mono uppercase tracking-wider text-[var(--text-muted)]">
                Ranked Recovery Options
              </span>
              <span className="text-[11px] font-mono text-[var(--accent)]">
                Deterministic Solver Preview
              </span>
            </div>

            {/* Candidate 1: Debt Repay */}
            <div className="p-4 rounded-xl border border-[var(--border)] bg-[var(--surface-2)]/40 hover:border-[var(--accent)]/40 transition-colors">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="text-xs font-mono font-bold px-1.5 py-0.5 rounded bg-[var(--accent)]/20 text-[var(--accent)]">
                    #1
                  </span>
                  <span className="text-sm font-semibold text-[var(--text-primary)]">
                    Repay Debt via Reserve
                  </span>
                </div>
                <span className="text-xs font-mono text-[var(--risk-safe)] font-semibold">
                  New HF: {hfAfterRepay}
                </span>
              </div>
              <p className="text-xs text-[var(--text-secondary)] mt-2">
                Repay {repayDebtAmount} {selectedScenario.borrowAsset} debt directly to elevate health factor safely above critical threshold.
              </p>
              <div className="mt-2.5 flex items-center gap-2 text-[11px] font-mono text-[var(--text-muted)]">
                <span>Feasibility: 98%</span>
                <span>•</span>
                <span>Slippage: 0.00%</span>
                <span>•</span>
                <span className="text-[var(--accent)]">No Swap Needed</span>
              </div>
            </div>

            {/* Candidate 2: Collateral Top-up or Swap */}
            <div className="p-4 rounded-xl border border-[var(--border)] bg-[var(--surface-2)]/40 hover:border-[var(--accent)]/40 transition-colors">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="text-xs font-mono font-bold px-1.5 py-0.5 rounded bg-[var(--surface-3)] text-[var(--text-muted)]">
                    #2
                  </span>
                  <span className="text-sm font-semibold text-[var(--text-primary)]">
                    Deposit Additional Collateral
                  </span>
                </div>
                <span className="text-xs font-mono text-[var(--risk-safe)] font-semibold">
                  New HF: {hfAfterAdd}
                </span>
              </div>
              <p className="text-xs text-[var(--text-secondary)] mt-2">
                Deposit +{addCollateralUnits} {selectedScenario.collateralAsset} to raise collateral backing without incurring trading fees.
              </p>
              <div className="mt-2.5 flex items-center gap-2 text-[11px] font-mono text-[var(--text-muted)]">
                <span>Feasibility: 92%</span>
                <span>•</span>
                <span>Indicative Jupiter Quote: N/A</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Advisory Notice */}
      <div className="mt-6 pt-4 border-t border-[var(--border)] flex items-center justify-between text-[11px] font-mono text-[var(--text-muted)]">
        <span>◈ Illustrative client scenario. Actual optimization is computed by Rust backend engines.</span>
        <span className="hidden sm:inline">Advisory only · Zero automated execution</span>
      </div>
    </div>
  );
}


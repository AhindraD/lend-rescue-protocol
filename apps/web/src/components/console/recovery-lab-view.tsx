"use client";

import { useEffect, useState, useMemo } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { listPositions, optimizeRecovery, getPositionRisk } from "@/lib/api/client";
import type {
  PositionResponse,
  RiskAssessmentResponse,
  RecoveryRecommendationResponse,
  RecoveryPlanDto,
} from "@/types/api";
import { ConsoleHeader } from "./header";
import { Card, CardBody, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton, SkeletonCard } from "@/components/ui/skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { EmptyState } from "@/components/ui/empty-state";
import { ProvenanceBadge } from "./provenance-badge";
import { truncateAddress } from "@/lib/utils";

export function RecoveryLabView() {
  const searchParams = useSearchParams();
  const initialPositionId = searchParams.get("positionId") || "";

  const [positions, setPositions] = useState<PositionResponse[]>([]);
  const [selectedPositionId, setSelectedPositionId] = useState<string>(initialPositionId);
  const [selectedRisk, setSelectedRisk] = useState<RiskAssessmentResponse | null>(null);
  const [targetHf, setTargetHf] = useState<string>("1.30");
  const [maxSlippageBps, setMaxSlippageBps] = useState<number>(100); // 1.00%

  const [loadingPositions, setLoadingPositions] = useState<boolean>(true);
  const [optimizing, setOptimizing] = useState<boolean>(false);
  const [recommendation, setRecommendation] = useState<RecoveryRecommendationResponse | null>(null);
  const [error, setError] = useState<{ message: string; code?: string; requestId?: string } | null>(null);
  const [multiAssetWarning, setMultiAssetWarning] = useState<string | null>(null);

  // Load positions on mount
  useEffect(() => {
    async function load() {
      setLoadingPositions(true);
      try {
        const list = await listPositions();
        setPositions(list);
        if (!selectedPositionId && list.length > 0) {
          setSelectedPositionId(list[0].id);
        }
      } catch (err: unknown) {
        if (err instanceof Error) {
          setError({
            message: err.message,
            code: (err as { code?: string }).code || "FETCH_FAILED",
          });
        }
      } finally {
        setLoadingPositions(false);
      }
    }
    load();
  }, []);

  // Update selected risk whenever selected position changes
  useEffect(() => {
    if (!selectedPositionId) {
      setSelectedRisk(null);
      return;
    }
    setMultiAssetWarning(null);
    setRecommendation(null);

    async function loadRisk() {
      try {
        const r = await getPositionRisk(selectedPositionId);
        setSelectedRisk(r);
      } catch {
        setSelectedRisk(null);
      }
    }
    loadRisk();
  }, [selectedPositionId]);

  const selectedPosition = useMemo(
    () => positions.find((p) => p.id === selectedPositionId),
    [positions, selectedPositionId]
  );

  // Run the recovery optimizer
  const handleOptimize = async () => {
    if (!selectedPositionId) return;

    setOptimizing(true);
    setError(null);
    setMultiAssetWarning(null);

    try {
      const res = await optimizeRecovery({
        positionId: selectedPositionId,
        targetHealthFactor: targetHf,
        maxSlippageBps,
      });
      setRecommendation(res);
    } catch (err: unknown) {
      if (err instanceof Error) {
        const status = (err as { status?: number }).status;
        const msg = err.message || "";

        // Check if multi-asset Kamino position fail-closed rule triggered
        if (status === 422 || msg.toLowerCase().includes("multi-asset") || msg.toLowerCase().includes("unsupported")) {
          setMultiAssetWarning(
            msg ||
              "Recovery optimization is unsupported for multi-asset Kamino positions until multi-leg recovery math is implemented. Rebalancing multi-asset obligations requires individual reserve operations to prevent unintended liquidation exposure."
          );
        } else {
          setError({
            message: err.message,
            code: (err as { code?: string }).code || `HTTP_${status || 500}`,
            requestId: (err as { requestId?: string }).requestId,
          });
        }
      } else {
        setError({ message: "An unexpected error occurred during optimization" });
      }
    } finally {
      setOptimizing(false);
    }
  };

  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title="Recovery Strategy Lab"
        subtitle="Deterministic Recovery Candidate Modeling & Indicative Swap Analysis"
        provenanceLabel={selectedPosition?.provenanceLabel || "Live"}
        actions={
          <div className="flex items-center gap-2">
            <span className="text-xs font-mono text-[var(--risk-safe)] flex items-center gap-1.5">
              <span className="h-1.5 w-1.5 rounded-full bg-[var(--risk-safe)]" />
              Non-Custodial
            </span>
          </div>
        }
      />

      <div className="flex-1 p-6 space-y-6 max-w-7xl w-full">
        {/* Error Banner */}
        {error && (
          <ErrorState
            title="Recovery Optimization Error"
            message={error.message}
            code={error.code}
            requestId={error.requestId}
            onRetry={handleOptimize}
          />
        )}

        {/* Multi-Asset Fail-Closed Guard Banner */}
        {multiAssetWarning && (
          <div className="p-5 rounded-2xl border border-[var(--risk-warning)]/40 bg-[var(--risk-warning)]/10 flex items-start gap-4">
            <span className="text-xl font-bold text-[var(--risk-warning)] leading-none mt-0.5">
              ⚠
            </span>
            <div className="space-y-1">
              <h4 className="text-sm font-bold text-[var(--risk-warning)]">
                Multi-Asset Position Guard: Fail-Closed Protection
              </h4>
              <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
                {multiAssetWarning}
              </p>
              <div className="pt-2 text-[11px] font-mono text-[var(--text-muted)] flex items-center gap-2">
                <span>Status: HTTP 422 Unprocessable Entity</span>
                <span>•</span>
                <span>Enforced by Rust domain safety boundary</span>
              </div>
            </div>
          </div>
        )}

        {/* Top Controls Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Position Selector & Current Metrics */}
          <Card className="lg:col-span-5">
            <CardHeader>
              <div>
                <CardTitle>Select Monitored Position</CardTitle>
                <CardDescription>Obligations loaded from Axum API</CardDescription>
              </div>
            </CardHeader>
            <CardBody className="space-y-4">
              {loadingPositions ? (
                <Skeleton className="h-10 w-full" />
              ) : (
                <div>
                  <label className="text-xs font-mono uppercase text-[var(--text-muted)] block mb-1.5">
                    Position Account ID
                  </label>
                  <select
                    value={selectedPositionId}
                    onChange={(e) => setSelectedPositionId(e.target.value)}
                    className="w-full px-3 py-2 rounded-lg border border-[var(--border)] bg-[var(--surface-2)] text-xs text-[var(--text-primary)] font-mono focus:outline-none focus:border-[var(--accent)]"
                  >
                    {positions.map((p) => (
                      <option key={p.id} value={p.id}>
                        {truncateAddress(p.id, 8)} ({p.collateralAsset} / {p.debtAsset})
                      </option>
                    ))}
                  </select>
                </div>
              )}

              {selectedPosition && (
                <div className="p-3 rounded-lg bg-[var(--surface-2)]/60 border border-[var(--border)] space-y-2 text-xs font-mono">
                  <div className="flex justify-between">
                    <span className="text-[var(--text-muted)]">Collateral:</span>
                    <span className="text-[var(--text-primary)] font-semibold">
                      {selectedPosition.collateralAmount.formatted} {selectedPosition.collateralAsset}
                      {selectedRisk && ` ($${parseFloat(selectedRisk.collateralValueFormatted).toFixed(2)})`}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-[var(--text-muted)]">Debt:</span>
                    <span className="text-[var(--text-primary)] font-semibold">
                      {selectedPosition.debtAmount.formatted} {selectedPosition.debtAsset}
                      {selectedRisk && ` ($${parseFloat(selectedRisk.debtValueFormatted).toFixed(2)})`}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-[var(--text-muted)]">Current Health Factor:</span>
                    <span className="text-[var(--risk-critical)] font-bold">
                      {selectedRisk?.healthFactor || "—"}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-[var(--text-muted)]">Liquidation Price:</span>
                    <span className="text-[var(--risk-warning)]">
                      {selectedRisk?.liquidationPrice ? `$${parseFloat(selectedRisk.liquidationPrice.formatted).toFixed(2)}` : "—"}
                    </span>
                  </div>
                </div>
              )}
            </CardBody>
          </Card>

          {/* Target Health Factor & Optimizer Parameters */}
          <Card className="lg:col-span-7">
            <CardHeader>
              <div>
                <CardTitle>Optimizer Parameters</CardTitle>
                <CardDescription>Specify recovery target and execution constraints</CardDescription>
              </div>
            </CardHeader>
            <CardBody className="space-y-5">
              {/* Target Health Factor Selection */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <label className="text-xs font-mono uppercase text-[var(--text-muted)]">
                    Target Post-Recovery Health Factor
                  </label>
                  <span className="text-sm font-mono font-bold text-[var(--accent)]">
                    {targetHf}
                  </span>
                </div>

                <div className="grid grid-cols-3 gap-2 mb-3">
                  {[
                    { label: "1.25 (Safe)", val: "1.25" },
                    { label: "1.30 (Recommended)", val: "1.30" },
                    { label: "1.40 (Conservative)", val: "1.40" },
                  ].map((preset) => (
                    <button
                      key={preset.val}
                      onClick={() => setTargetHf(preset.val)}
                      className={`px-3 py-2 rounded-lg text-xs font-mono transition-colors cursor-pointer text-center ${
                        targetHf === preset.val
                          ? "bg-[var(--accent)]/15 text-[var(--accent)] border border-[var(--accent)]/40 font-bold"
                          : "bg-[var(--surface-2)] text-[var(--text-muted)] border border-[var(--border)] hover:text-[var(--text-primary)]"
                      }`}
                    >
                      {preset.label}
                    </button>
                  ))}
                </div>

                <input
                  type="range"
                  min="1.15"
                  max="1.60"
                  step="0.05"
                  value={targetHf}
                  onChange={(e) => setTargetHf(e.target.value)}
                  className="w-full accent-[var(--accent)] cursor-pointer"
                />
              </div>

              {/* Slippage Tolerance Selector */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <label className="text-xs font-mono uppercase text-[var(--text-muted)]">
                    Max Slippage Tolerance (Jupiter DEX Route)
                  </label>
                  <span className="text-xs font-mono text-[var(--text-secondary)]">
                    {(maxSlippageBps / 100).toFixed(2)}% ({maxSlippageBps} bps)
                  </span>
                </div>

                <div className="flex gap-2">
                  {[50, 100, 200].map((bps) => (
                    <button
                      key={bps}
                      onClick={() => setMaxSlippageBps(bps)}
                      className={`px-3 py-1.5 rounded-lg text-xs font-mono transition-colors cursor-pointer ${
                        maxSlippageBps === bps
                          ? "bg-[var(--surface-3)] text-[var(--text-primary)] border border-[var(--border)] font-semibold"
                          : "bg-[var(--surface-2)] text-[var(--text-muted)] border border-[var(--border)]"
                      }`}
                    >
                      {(bps / 100).toFixed(2)}%
                    </button>
                  ))}
                </div>
              </div>

              {/* Action Trigger */}
              <div className="pt-2 flex items-center justify-between">
                <span className="text-[11px] font-mono text-[var(--text-muted)]">
                  Evaluated deterministically by pure Rust optimizer
                </span>
                <Button
                  variant="primary"
                  size="md"
                  onClick={handleOptimize}
                  loading={optimizing}
                  disabled={!selectedPositionId}
                >
                  Run Optimizer →
                </Button>
              </div>
            </CardBody>
          </Card>
        </div>

        {/* Optimizing State */}
        {optimizing && (
          <div className="space-y-4">
            <div className="flex items-center gap-3 text-sm font-mono text-[var(--accent)]">
              <span className="h-2 w-2 rounded-full bg-[var(--accent)] animate-pulse" />
              <span>Evaluating candidate combinations and requesting indicative Jupiter quotes...</span>
            </div>
            <SkeletonCard />
            <SkeletonCard />
          </div>
        )}

        {/* Ranked Recovery Candidates */}
        {recommendation && !optimizing && (
          <div className="space-y-6">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-[var(--border)] pb-4">
              <div>
                <h3 className="text-lg font-bold text-[var(--text-primary)]">
                  Ranked Recovery Proposals
                </h3>
                <p className="text-xs text-[var(--text-muted)] mt-0.5">
                  Proposals ranked in exact order returned by the optimizer. Pre-HF: {recommendation.preHealthFactor} → Target: {recommendation.targetHealthFactor}
                </p>
              </div>
              <span className="text-xs font-mono text-[var(--text-muted)]">
                Recommendation ID: {truncateAddress(recommendation.recoveryId, 6)}
              </span>
            </div>

            {/* List all candidates in returned order */}
            {(() => {
              const allPlans: { plan: RecoveryPlanDto; isTop: boolean; rank: number }[] = [];
              if (recommendation.recommendedPlan) {
                allPlans.push({ plan: recommendation.recommendedPlan, isTop: true, rank: 1 });
              }
              recommendation.alternativePlans.forEach((plan, idx) => {
                allPlans.push({ plan, isTop: false, rank: idx + (recommendation.recommendedPlan ? 2 : 1) });
              });

              if (allPlans.length === 0) {
                return (
                  <EmptyState
                    title="No feasible recovery plans found"
                    description="The optimizer could not identify any combinations achieving the target health factor within the specified constraints."
                  />
                );
              }

              return allPlans.map(({ plan, isTop, rank }) => (
                <Card
                  key={rank}
                  className={`border ${
                    isTop
                      ? "border-[var(--accent)]/50 bg-[var(--surface-1)] shadow-lg shadow-[var(--accent)]/5"
                      : "border-[var(--border)] bg-[var(--surface-1)]"
                  }`}
                >
                  <CardHeader className="flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
                    <div className="flex items-center gap-3">
                      <span
                        className={`px-2 py-0.5 rounded text-xs font-mono font-bold ${
                          isTop
                            ? "bg-[var(--accent)] text-[var(--background)]"
                            : "bg-[var(--surface-3)] text-[var(--text-secondary)] border border-[var(--border)]"
                        }`}
                      >
                        #{rank} {isTop ? "RECOMMENDED" : "ALTERNATIVE"}
                      </span>
                      <h4 className="text-base font-bold text-[var(--text-primary)]">
                        {plan.actionLabel}
                      </h4>
                    </div>

                    <div className="flex items-center gap-3 font-mono text-xs">
                      <span className="text-[var(--text-muted)]">Post-Recovery HF:</span>
                      <span className="text-sm font-bold text-[var(--risk-safe)]">
                        {plan.postHealthFactor}
                      </span>
                      <Badge variant={plan.isFeasible ? "safe" : "critical"}>
                        {plan.isFeasible ? "Feasible" : "Infeasible"}
                      </Badge>
                    </div>
                  </CardHeader>

                  <CardBody className="space-y-5">
                    {/* Key Proposal Financials */}
                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 p-4 rounded-xl bg-[var(--surface-2)]/60 border border-[var(--border)] font-mono text-xs">
                      <div>
                        <p className="text-[11px] uppercase text-[var(--text-muted)]">Debt Repaid</p>
                        <p className="text-sm font-semibold text-[var(--text-primary)] mt-0.5">
                          {plan.debtRepaid.formatted} {plan.debtRepaid.symbol || ""}
                        </p>
                      </div>
                      <div>
                        <p className="text-[11px] uppercase text-[var(--text-muted)]">Collateral Spent</p>
                        <p className="text-sm font-semibold text-[var(--text-primary)] mt-0.5">
                          {plan.collateralSpent.formatted} {plan.collateralSpent.symbol || ""}
                        </p>
                      </div>
                      <div>
                        <p className="text-[11px] uppercase text-[var(--text-muted)]">Expected Fee</p>
                        <p className="text-sm font-semibold text-[var(--text-primary)] mt-0.5">
                          ${plan.expectedFeeFormatted}
                        </p>
                      </div>
                      <div>
                        <p className="text-[11px] uppercase text-[var(--text-muted)]">Est. Slippage</p>
                        <p className="text-sm font-semibold text-[var(--text-primary)] mt-0.5">
                          {(plan.estimatedSlippageBps / 100).toFixed(2)}%
                        </p>
                      </div>
                    </div>

                    {/* Score Breakdown */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between text-xs font-mono">
                        <span className="text-[var(--text-muted)] uppercase">Optimizer Score Breakdown</span>
                        <span className="text-[var(--accent)] font-bold">Total Score: {plan.score}</span>
                      </div>
                      <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs font-mono">
                        <div className="p-2 rounded bg-[var(--surface-2)] border border-[var(--border)]">
                          <span className="text-[var(--text-muted)] block text-[10px]">Capital Efficiency</span>
                          <span className="font-semibold text-[var(--text-primary)]">{plan.scoreBreakdown.capitalEfficiency}</span>
                        </div>
                        <div className="p-2 rounded bg-[var(--surface-2)] border border-[var(--border)]">
                          <span className="text-[var(--text-muted)] block text-[10px]">Safety Buffer</span>
                          <span className="font-semibold text-[var(--text-primary)]">{plan.scoreBreakdown.safetyBuffer}</span>
                        </div>
                        <div className="p-2 rounded bg-[var(--surface-2)] border border-[var(--border)]">
                          <span className="text-[var(--text-muted)] block text-[10px]">Fee Penalty</span>
                          <span className="font-semibold text-[var(--risk-warning)]">-{plan.scoreBreakdown.feePenalty}</span>
                        </div>
                        <div className="p-2 rounded bg-[var(--surface-2)] border border-[var(--border)]">
                          <span className="text-[var(--text-muted)] block text-[10px]">Slippage Penalty</span>
                          <span className="font-semibold text-[var(--risk-warning)]">-{plan.scoreBreakdown.slippagePenalty}</span>
                        </div>
                      </div>
                    </div>

                    {/* Indicative Jupiter Swap Quote Card */}
                    {plan.swapQuote && (
                      <div className="p-4 rounded-xl border border-[var(--border)] bg-[var(--surface-2)]/80 space-y-3">
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-2">
                            <span className="h-2 w-2 rounded-full bg-[var(--risk-warning)]" />
                            <span className="text-xs font-mono font-bold uppercase text-[var(--text-primary)]">
                              Indicative Jupiter quote
                            </span>
                          </div>
                          <span className="text-[11px] font-mono text-[var(--text-muted)]">
                            Quote Provider: {plan.swapQuote.provider}
                          </span>
                        </div>

                        {/* Swap Routing Details */}
                        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs font-mono">
                          <div className="p-2.5 rounded bg-[var(--surface-1)] border border-[var(--border)]">
                            <span className="text-[var(--text-muted)] block text-[10px]">Input Amount</span>
                            <span className="font-semibold text-[var(--text-primary)]">
                              {plan.swapQuote.inAmount.formatted} {plan.swapQuote.inAmount.symbol || "tokens"}
                            </span>
                          </div>
                          <div className="p-2.5 rounded bg-[var(--surface-1)] border border-[var(--border)]">
                            <span className="text-[var(--text-muted)] block text-[10px]">Expected Output</span>
                            <span className="font-semibold text-[var(--risk-safe)]">
                              {plan.swapQuote.expectedOutAmount.formatted} {plan.swapQuote.expectedOutAmount.symbol || "tokens"}
                            </span>
                          </div>
                          <div className="p-2.5 rounded bg-[var(--surface-1)] border border-[var(--border)]">
                            <span className="text-[var(--text-muted)] block text-[10px]">Price Impact</span>
                            <span className="font-semibold text-[var(--text-primary)]">
                              {plan.swapQuote.priceImpactPct ? `${parseFloat(plan.swapQuote.priceImpactPct).toFixed(2)}%` : "< 0.05%"}
                            </span>
                          </div>
                        </div>

                        {/* Multi-Hop Route Steps */}
                        {plan.swapQuote.routePlan && plan.swapQuote.routePlan.length > 0 && (
                          <div className="pt-2">
                            <p className="text-[11px] font-mono uppercase text-[var(--text-muted)] mb-1.5">
                              Route Plan Execution Steps
                            </p>
                            <div className="flex flex-wrap items-center gap-2">
                              {plan.swapQuote.routePlan.map((step, sIdx) => (
                                <div
                                  key={sIdx}
                                  className="px-2.5 py-1 rounded bg-[var(--surface-1)] border border-[var(--border)] text-[11px] font-mono text-[var(--text-secondary)] flex items-center gap-1.5"
                                >
                                  <span className="text-[var(--accent)] font-bold">{step.label || "DEX Pool"}</span>
                                  <span className="text-[var(--text-muted)]">({step.percent}%)</span>
                                </div>
                              ))}
                            </div>
                          </div>
                        )}

                        <p className="text-[11px] font-mono text-[var(--text-muted)] italic pt-1">
                          Snapshot quote from Jupiter Swap API V2. Strictly indicative for decision planning; no swap transaction constructed.
                        </p>
                      </div>
                    )}
                  </CardBody>
                </Card>
              ));
            })()}

            {/* Advisory Guarantee Banner */}
            <div className="p-4 rounded-xl border border-[var(--border)] bg-[var(--surface-2)]/30 text-xs font-mono text-[var(--text-muted)] flex flex-col sm:flex-row items-center justify-between gap-3">
              <span>◈ Non-custodial advisory recommendation. LendRescue does not construct or broadcast transactions.</span>
              <span className="text-[var(--risk-safe)] font-semibold shrink-0">Deterministic Solver</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}


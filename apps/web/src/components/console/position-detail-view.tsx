"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { getPosition, getPositionRisk } from "@/lib/api/client";
import type { PositionResponse, RiskAssessmentResponse } from "@/types/api";
import { ConsoleHeader } from "./header";
import { Card, CardBody, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton, SkeletonCard } from "@/components/ui/skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { ProvenanceBadge } from "./provenance-badge";
import { truncateAddress } from "@/lib/utils";

interface PositionDetailViewProps {
  positionId: string;
}

export function PositionDetailView({ positionId }: PositionDetailViewProps) {
  const [position, setPosition] = useState<PositionResponse | null>(null);
  const [risk, setRisk] = useState<RiskAssessmentResponse | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<{ message: string; code?: string; requestId?: string } | null>(null);

  const fetchDetail = async () => {
    setLoading(true);
    setError(null);
    try {
      const pos = await getPosition(positionId);
      setPosition(pos);
      try {
        const r = await getPositionRisk(positionId);
        setRisk(r);
      } catch {
        // Position risk could be unsupported for multi-asset
      }
    } catch (err: unknown) {
      if (err instanceof Error) {
        setError({
          message: err.message,
          code: (err as { code?: string }).code || "NOT_FOUND",
          requestId: (err as { requestId?: string }).requestId,
        });
      } else {
        setError({ message: "Failed to load position details" });
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (positionId) {
      fetchDetail();
    }
  }, [positionId]);

  const isMultiAsset =
    (position?.collateralPositions && position.collateralPositions.length > 1) ||
    (position?.debtPositions && position.debtPositions.length > 1);

  const riskBand = risk?.riskBand || "Unknown";
  let badgeVariant: "safe" | "warning" | "critical" | "muted" = "muted";
  let hfClass = "text-[var(--text-muted)]";

  if (riskBand.toLowerCase() === "critical") {
    badgeVariant = "critical";
    hfClass = "text-[var(--risk-critical)] font-bold";
  } else if (riskBand.toLowerCase() === "warning" || riskBand.toLowerCase() === "moderate") {
    badgeVariant = "warning";
    hfClass = "text-[var(--risk-warning)] font-bold";
  } else if (riskBand.toLowerCase() === "safe") {
    badgeVariant = "safe";
    hfClass = "text-[var(--risk-safe)] font-bold";
  }

  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title={`Position Inspection: ${truncateAddress(positionId, 6)}`}
        subtitle="On-Chain Account Architecture & Real-Time Risk Verification"
        provenanceLabel={position?.provenanceLabel || "Live"}
        actions={
          <div className="flex items-center gap-2">
            <Link
              href="/app/positions"
              className="px-3 py-1.5 rounded-lg border border-[var(--border)] bg-[var(--surface-2)] text-xs font-mono text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            >
              ← Back to Positions
            </Link>
            <Link
              href={`/app/recovery?positionId=${encodeURIComponent(positionId)}`}
              className="px-3 py-1.5 rounded-lg bg-[var(--accent)] text-[var(--background)] text-xs font-mono font-bold hover:bg-[var(--accent)]/90 transition-colors shadow-sm"
            >
              Open Recovery Lab →
            </Link>
          </div>
        }
      />

      <div className="flex-1 p-6 space-y-6 max-w-7xl w-full">
        {error && (
          <ErrorState
            title="Position Lookup Failed"
            message={error.message}
            code={error.code}
            requestId={error.requestId}
            onRetry={fetchDetail}
          />
        )}

        {loading && !position && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {Array.from({ length: 3 }).map((_, i) => (
                <SkeletonCard key={i} />
              ))}
            </div>
            <Skeleton className="h-72 w-full rounded-xl" />
          </div>
        )}

        {position && (
          <>
            {/* Multi-Asset Guard Warning Banner */}
            {isMultiAsset && (
              <div className="p-4 rounded-xl border border-[var(--risk-warning)]/30 bg-[var(--risk-warning)]/10 flex items-start gap-3">
                <span className="text-[var(--risk-warning)] font-bold text-lg leading-none">⚠</span>
                <div>
                  <h4 className="text-sm font-bold text-[var(--risk-warning)]">
                    Multi-Asset Obligation Detected
                  </h4>
                  <p className="text-xs text-[var(--text-secondary)] mt-1 leading-relaxed">
                    This position holds multiple collateral deposits or debt borrows. To maintain mathematical safety,
                    LendRescue enforces fail-closed isolation. Cross-currency synthetic rebalancing must be evaluated
                    per isolated reserve asset to prevent liquidation underestimation.
                  </p>
                </div>
              </div>
            )}

            {/* Core Snapshot Cards */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <Card>
                <CardHeader>
                  <CardTitle>Health Factor &amp; Risk Band</CardTitle>
                  <Badge variant={badgeVariant}>{risk?.riskBandLabel || riskBand}</Badge>
                </CardHeader>
                <CardBody>
                  <div className="flex items-baseline gap-2 mb-2">
                    <span className={`text-4xl font-extrabold font-mono ${hfClass}`}>
                      {risk?.healthFactor || "—"}
                    </span>
                  </div>
                  <div className="space-y-1.5 text-xs font-mono text-[var(--text-secondary)]">
                    <div className="flex justify-between">
                      <span className="text-[var(--text-muted)]">Liquidation LT:</span>
                      <span>{(position.liquidationThresholdBps / 100).toFixed(1)}%</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-[var(--text-muted)]">Liq Price:</span>
                      <span className="text-[var(--risk-warning)]">
                        {risk?.liquidationPrice ? `$${parseFloat(risk.liquidationPrice.formatted).toFixed(2)}` : "—"}
                      </span>
                    </div>
                  </div>
                </CardBody>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Total Collateral Backing</CardTitle>
                  <ProvenanceBadge label={position.provenanceLabel} showAge={false} />
                </CardHeader>
                <CardBody>
                  <p className="text-2xl font-bold font-mono text-[var(--text-primary)]">
                    {position.collateralAmount.formatted} {position.collateralAsset}
                  </p>
                  <p className="text-xs font-mono text-[var(--text-secondary)] mt-1">
                    {risk ? `$${parseFloat(risk.collateralValueFormatted).toLocaleString(undefined, { minimumFractionDigits: 2 })}` : "—"}
                  </p>
                  <div className="mt-3 pt-3 border-t border-[var(--border)] text-xs font-mono text-[var(--text-muted)]">
                    Protocol: {position.protocolSource}
                  </div>
                </CardBody>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Outstanding Debt Exposure</CardTitle>
                  <span className="text-xs font-mono text-[var(--risk-critical)]">Active Borrow</span>
                </CardHeader>
                <CardBody>
                  <p className="text-2xl font-bold font-mono text-[var(--text-primary)]">
                    {position.debtAmount.formatted} {position.debtAsset}
                  </p>
                  <p className="text-xs font-mono text-[var(--text-secondary)] mt-1">
                    {risk ? `$${parseFloat(risk.debtValueFormatted).toLocaleString(undefined, { minimumFractionDigits: 2 })}` : "—"}
                  </p>
                  <div className="mt-3 pt-3 border-t border-[var(--border)] text-xs font-mono text-[var(--text-muted)]">
                    Borrow Capacity: {risk ? `$${parseFloat(risk.borrowCapacityFormatted).toLocaleString(undefined, { minimumFractionDigits: 2 })}` : "—"}
                  </div>
                </CardBody>
              </Card>
            </div>

            {/* Protocol Account Architecture & Reserves */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Account Keys */}
              <Card>
                <CardHeader>
                  <CardTitle>On-Chain Identifiers</CardTitle>
                  <CardDescription>Verified Solana SVM Public Keys</CardDescription>
                </CardHeader>
                <CardBody className="space-y-3 font-mono text-xs">
                  <div>
                    <span className="text-[var(--text-muted)] block mb-1">Obligation Account</span>
                    <span className="p-2 rounded bg-[var(--surface-2)] border border-[var(--border)] block text-[var(--text-primary)] select-all break-all">
                      {position.id}
                    </span>
                  </div>
                  <div>
                    <span className="text-[var(--text-muted)] block mb-1">Owner Authority</span>
                    <span className="p-2 rounded bg-[var(--surface-2)] border border-[var(--border)] block text-[var(--text-primary)] select-all break-all">
                      {position.owner}
                    </span>
                  </div>
                  {position.marketAddress && (
                    <div>
                      <span className="text-[var(--text-muted)] block mb-1">Kamino Lending Market</span>
                      <span className="p-2 rounded bg-[var(--surface-2)] border border-[var(--border)] block text-[var(--text-primary)] select-all break-all">
                        {position.marketAddress}
                      </span>
                    </div>
                  )}
                </CardBody>
              </Card>

              {/* Protocol Reported Risk Metrics */}
              <Card>
                <CardHeader>
                  <CardTitle>Protocol Reported Risk Metrics</CardTitle>
                  <CardDescription>Decoded directly from Kamino state</CardDescription>
                </CardHeader>
                <CardBody>
                  {position.protocolMetrics ? (
                    <div className="space-y-2.5 font-mono text-xs">
                      <div className="flex justify-between py-1 border-b border-[var(--border)]">
                        <span className="text-[var(--text-muted)]">Borrow Limit USD</span>
                        <span className="text-[var(--text-primary)] font-semibold">
                          ${parseFloat(position.protocolMetrics.borrowLimitUsd).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                        </span>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[var(--border)]">
                        <span className="text-[var(--text-muted)]">Liquidation Threshold USD</span>
                        <span className="text-[var(--text-primary)] font-semibold">
                          ${parseFloat(position.protocolMetrics.liquidationThresholdUsd).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                        </span>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[var(--border)]">
                        <span className="text-[var(--text-muted)]">Loan-to-Value (LTV)</span>
                        <span className="text-[var(--text-primary)] font-semibold">
                          {(position.protocolMetrics.loanToValueBps / 100).toFixed(2)}%
                        </span>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[var(--border)]">
                        <span className="text-[var(--text-muted)]">Borrow Utilization</span>
                        <span className="text-[var(--text-primary)] font-semibold">
                          {(position.protocolMetrics.borrowUtilizationBps / 100).toFixed(2)}%
                        </span>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[var(--border)]">
                        <span className="text-[var(--text-muted)]">Net Account Value</span>
                        <span className="text-[var(--risk-safe)] font-semibold">
                          ${parseFloat(position.protocolMetrics.netAccountValueUsd).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                        </span>
                      </div>
                    </div>
                  ) : (
                    <p className="text-xs font-mono text-[var(--text-muted)]">
                      Synthetic or single-reserve position. Protocol aggregate metrics not reported.
                    </p>
                  )}
                </CardBody>
              </Card>
            </div>

            {/* Quick Action Banner */}
            <div className="p-6 rounded-2xl border border-[var(--border)] bg-gradient-to-r from-[var(--surface-1)] to-[var(--surface-2)] flex flex-col sm:flex-row items-center justify-between gap-4">
              <div>
                <h3 className="text-base font-bold text-[var(--text-primary)]">
                  Simulate Recovery Options in Strategy Lab
                </h3>
                <p className="text-xs text-[var(--text-secondary)] mt-1">
                  Run our deterministic recovery optimizer to rank debt repayments, collateral swaps, and indicative Jupiter quotes.
                </p>
              </div>
              <Link
                href={`/app/recovery?positionId=${encodeURIComponent(positionId)}`}
                className="px-6 py-2.5 rounded-lg bg-[var(--accent)] text-[var(--background)] text-sm font-mono font-bold hover:bg-[var(--accent)]/90 transition-all shrink-0 shadow-md"
              >
                Model Recovery Now →
              </Link>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

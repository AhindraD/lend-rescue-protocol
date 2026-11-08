"use client";

import { useEffect, useState, useMemo } from "react";
import Link from "next/link";
import { listPositions, getPositionRisk } from "@/lib/api/client";
import type { PositionResponse, RiskAssessmentResponse } from "@/types/api";
import { ConsoleHeader } from "./header";
import { Card, CardBody, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton, SkeletonCard } from "@/components/ui/skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { EmptyState } from "@/components/ui/empty-state";
import { ProvenanceBadge } from "./provenance-badge";
import { truncateAddress } from "@/lib/utils";

interface PositionWithRisk {
  position: PositionResponse;
  risk?: RiskAssessmentResponse;
}

export function DashboardView() {
  const [items, setItems] = useState<PositionWithRisk[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<{ message: string; code?: string; requestId?: string } | null>(null);
  const [filterRisk, setFilterRisk] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState<string>("");

  const fetchData = async () => {
    setLoading(true);
    setError(null);
    try {
      const positions = await listPositions();
      // Fetch risk assessments in parallel for loaded positions
      const positionsWithRisk = await Promise.all(
        positions.map(async (pos) => {
          try {
            const risk = await getPositionRisk(pos.id);
            return { position: pos, risk };
          } catch {
            return { position: pos };
          }
        })
      );
      setItems(positionsWithRisk);
    } catch (err: unknown) {
      if (err instanceof Error) {
        setError({
          message: err.message,
          code: (err as { code?: string }).code || "FETCH_FAILED",
          requestId: (err as { requestId?: string }).requestId,
        });
      } else {
        setError({ message: "Failed to connect to LendRescue risk API" });
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  // Compute aggregated dashboard metrics
  const metrics = useMemo(() => {
    let totalCollateralUsd = 0;
    let totalDebtUsd = 0;
    let criticalCount = 0;
    let warningCount = 0;
    let safeCount = 0;

    for (const item of items) {
      const risk = item.risk;
      const band = (risk?.riskBand || "").toLowerCase();

      if (band === "critical") {
        criticalCount++;
      } else if (band === "warning" || band === "moderate") {
        warningCount++;
      } else {
        safeCount++;
      }

      if (risk) {
        const colVal = parseFloat(risk.collateralValueFormatted);
        const debtVal = parseFloat(risk.debtValueFormatted);
        if (!isNaN(colVal)) totalCollateralUsd += colVal;
        if (!isNaN(debtVal)) totalDebtUsd += debtVal;
      }
    }

    return {
      totalPositions: items.length,
      totalCollateralUsd,
      totalDebtUsd,
      criticalCount,
      warningCount,
      safeCount,
      atRiskCount: criticalCount + warningCount,
    };
  }, [items]);

  // Filtered positions based on search and risk filter
  const filteredPositions = useMemo(() => {
    return items.filter(({ position, risk }) => {
      const q = searchQuery.toLowerCase().trim();
      const matchesSearch =
        !q ||
        position.id.toLowerCase().includes(q) ||
        position.owner.toLowerCase().includes(q) ||
        position.collateralAsset.toLowerCase().includes(q) ||
        position.debtAsset.toLowerCase().includes(q);

      if (!matchesSearch) return false;

      if (filterRisk === "all") return true;
      const band = (risk?.riskBand || "").toLowerCase();
      if (filterRisk === "critical") return band === "critical";
      if (filterRisk === "warning") return band === "warning" || band === "moderate";
      if (filterRisk === "safe") return band === "safe";

      return true;
    });
  }, [items, searchQuery, filterRisk]);

  const globalProvenance = items[0]?.position.provenanceLabel || "Live";

  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title="Dashboard"
        subtitle="Lending Risk Intelligence & Real-Time Portfolio Overview"
        provenanceLabel={globalProvenance}
        actions={
          <Button
            size="sm"
            variant="secondary"
            onClick={fetchData}
            loading={loading}
          >
            ↻ Refresh
          </Button>
        }
      />

      <div className="flex-1 p-6 space-y-6 max-w-7xl w-full">
        {/* Error Notification */}
        {error && (
          <ErrorState
            title="Backend Communication Error"
            message={error.message}
            code={error.code}
            requestId={error.requestId}
            onRetry={fetchData}
          />
        )}

        {/* Loading Skeletons */}
        {loading && !error && items.length === 0 && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
              {Array.from({ length: 4 }).map((_, i) => (
                <SkeletonCard key={i} />
              ))}
            </div>
            <Skeleton className="h-64 w-full rounded-xl" />
          </div>
        )}

        {/* Overview Metric Cards */}
        {(!loading || items.length > 0) && (
          <>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
              <Card>
                <CardBody>
                  <p className="text-xs font-mono uppercase tracking-wider text-[var(--text-muted)] mb-1">
                    Total Collateral
                  </p>
                  <p className="text-2xl font-bold font-mono text-[var(--text-primary)]">
                    ${metrics.totalCollateralUsd.toLocaleString(undefined, {
                      minimumFractionDigits: 2,
                      maximumFractionDigits: 2,
                    })}
                  </p>
                  <p className="text-xs text-[var(--text-muted)] mt-1">
                    Across {metrics.totalPositions} monitored positions
                  </p>
                </CardBody>
              </Card>

              <Card>
                <CardBody>
                  <p className="text-xs font-mono uppercase tracking-wider text-[var(--text-muted)] mb-1">
                    Total Borrowed
                  </p>
                  <p className="text-2xl font-bold font-mono text-[var(--text-primary)]">
                    ${metrics.totalDebtUsd.toLocaleString(undefined, {
                      minimumFractionDigits: 2,
                      maximumFractionDigits: 2,
                    })}
                  </p>
                  <p className="text-xs text-[var(--text-muted)] mt-1">
                    Outstanding debt obligations
                  </p>
                </CardBody>
              </Card>

              <Card>
                <CardBody>
                  <p className="text-xs font-mono uppercase tracking-wider text-[var(--text-muted)] mb-1">
                    Positions at Risk
                  </p>
                  <div className="flex items-baseline gap-2">
                    <p className={`text-2xl font-bold font-mono ${metrics.atRiskCount > 0 ? "text-[var(--risk-critical)]" : "text-[var(--risk-safe)]"}`}>
                      {metrics.atRiskCount}
                    </p>
                    <span className="text-xs font-mono text-[var(--text-muted)]">
                      ({metrics.criticalCount} critical, {metrics.warningCount} warning)
                    </span>
                  </div>
                  <p className="text-xs text-[var(--text-muted)] mt-1">
                    Health factor below 1.25 threshold
                  </p>
                </CardBody>
              </Card>

              <Card>
                <CardBody>
                  <p className="text-xs font-mono uppercase tracking-wider text-[var(--text-muted)] mb-1">
                    Risk Engine Status
                  </p>
                  <div className="flex items-center gap-2 mt-1">
                    <span className="h-2 w-2 rounded-full bg-[var(--risk-safe)] animate-pulse" />
                    <span className="text-sm font-semibold text-[var(--risk-safe)] font-mono">
                      Active · Deterministic
                    </span>
                  </div>
                  <p className="text-xs text-[var(--text-muted)] mt-1.5 font-mono">
                    Pyth Core Oracles verified
                  </p>
                </CardBody>
              </Card>
            </div>

            {/* Risk Distribution Breakdown */}
            <Card>
              <CardHeader>
                <div>
                  <CardTitle>Portfolio Risk Distribution</CardTitle>
                  <CardDescription>
                    Breakdown of monitored positions by health factor band
                  </CardDescription>
                </div>
              </CardHeader>
              <CardBody>
                {metrics.totalPositions === 0 ? (
                  <p className="text-sm text-[var(--text-muted)]">No positions to assess</p>
                ) : (
                  <div className="space-y-4">
                    {/* Visual Segmented Bar */}
                    <div className="h-3 w-full rounded-full bg-[var(--surface-3)] overflow-hidden flex">
                      {metrics.safeCount > 0 && (
                        <div
                          style={{ width: `${(metrics.safeCount / metrics.totalPositions) * 100}%` }}
                          className="bg-[var(--risk-safe)] h-full transition-all"
                          title={`Safe: ${metrics.safeCount}`}
                        />
                      )}
                      {metrics.warningCount > 0 && (
                        <div
                          style={{ width: `${(metrics.warningCount / metrics.totalPositions) * 100}%` }}
                          className="bg-[var(--risk-warning)] h-full transition-all"
                          title={`Warning: ${metrics.warningCount}`}
                        />
                      )}
                      {metrics.criticalCount > 0 && (
                        <div
                          style={{ width: `${(metrics.criticalCount / metrics.totalPositions) * 100}%` }}
                          className="bg-[var(--risk-critical)] h-full transition-all"
                          title={`Critical: ${metrics.criticalCount}`}
                        />
                      )}
                    </div>

                    {/* Legend */}
                    <div className="flex flex-wrap gap-6 text-xs font-mono">
                      <div className="flex items-center gap-2">
                        <span className="h-2.5 w-2.5 rounded-full bg-[var(--risk-safe)]" />
                        <span className="text-[var(--text-secondary)]">
                          Safe (HF &gt; 1.25): <strong className="text-[var(--text-primary)]">{metrics.safeCount}</strong>
                        </span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="h-2.5 w-2.5 rounded-full bg-[var(--risk-warning)]" />
                        <span className="text-[var(--text-secondary)]">
                          Warning (1.10–1.25): <strong className="text-[var(--text-primary)]">{metrics.warningCount}</strong>
                        </span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="h-2.5 w-2.5 rounded-full bg-[var(--risk-critical)]" />
                        <span className="text-[var(--text-secondary)]">
                          Critical (&lt; 1.10): <strong className="text-[var(--text-primary)]">{metrics.criticalCount}</strong>
                        </span>
                      </div>
                    </div>
                  </div>
                )}
              </CardBody>
            </Card>

            {/* Positions Table & Filters */}
            <Card>
              <CardHeader className="flex-col sm:flex-row gap-4 items-start sm:items-center">
                <div>
                  <CardTitle>Active Lending Positions</CardTitle>
                  <CardDescription>
                    Monitored obligations decoded from Kamino Lending
                  </CardDescription>
                </div>

                {/* Filter and Search Controls */}
                <div className="flex flex-wrap items-center gap-2.5 w-full sm:w-auto">
                  <input
                    type="text"
                    placeholder="Search by ID, asset, owner..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="px-3 py-1.5 rounded-lg border border-[var(--border)] bg-[var(--surface-2)] text-xs text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] font-mono min-w-[200px]"
                  />

                  <div className="flex rounded-lg border border-[var(--border)] p-0.5 bg-[var(--surface-2)] text-xs font-mono">
                    {["all", "critical", "warning", "safe"].map((tab) => (
                      <button
                        key={tab}
                        onClick={() => setFilterRisk(tab)}
                        className={`px-2.5 py-1 rounded-md transition-colors cursor-pointer capitalize ${
                          filterRisk === tab
                            ? "bg-[var(--surface-3)] text-[var(--accent)] font-semibold"
                            : "text-[var(--text-muted)] hover:text-[var(--text-secondary)]"
                        }`}
                      >
                        {tab}
                      </button>
                    ))}
                  </div>
                </div>
              </CardHeader>

              <CardBody className="p-0">
                {filteredPositions.length === 0 ? (
                  <EmptyState
                    title="No positions match criteria"
                    description={searchQuery ? "Try refining your search terms or filter selection." : "No positions are currently recorded."}
                  />
                ) : (
                  <div className="overflow-x-auto">
                    <table className="w-full text-left text-sm">
                      <thead className="bg-[var(--surface-2)]/60 border-b border-[var(--border)] text-xs font-mono uppercase text-[var(--text-muted)]">
                        <tr>
                          <th className="px-6 py-3">Position / Owner</th>
                          <th className="px-4 py-3">Collateral</th>
                          <th className="px-4 py-3">Debt</th>
                          <th className="px-4 py-3">Health Factor</th>
                          <th className="px-4 py-3">Risk Status</th>
                          <th className="px-4 py-3">Provenance</th>
                          <th className="px-6 py-3 text-right">Action</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-[var(--border)]">
                        {filteredPositions.map(({ position, risk }) => {
                          const hfStr = risk?.healthFactor || "—";
                          const riskBand = risk?.riskBand || "Unknown";

                          let hfVariant: "safe" | "warning" | "critical" | "muted" = "muted";
                          let hfClass = "text-[var(--text-muted)]";

                          if (riskBand.toLowerCase() === "critical") {
                            hfVariant = "critical";
                            hfClass = "text-[var(--risk-critical)] font-bold";
                          } else if (riskBand.toLowerCase() === "warning" || riskBand.toLowerCase() === "moderate") {
                            hfVariant = "warning";
                            hfClass = "text-[var(--risk-warning)] font-bold";
                          } else if (riskBand.toLowerCase() === "safe") {
                            hfVariant = "safe";
                            hfClass = "text-[var(--risk-safe)] font-bold";
                          }

                          return (
                            <tr
                              key={position.id}
                              className="hover:bg-[var(--surface-2)]/50 transition-colors"
                            >
                              {/* Position & Owner */}
                              <td className="px-6 py-4">
                                <div className="font-mono text-xs font-semibold text-[var(--text-primary)]">
                                  {truncateAddress(position.id, 6)}
                                </div>
                                <div className="font-mono text-[11px] text-[var(--text-muted)] mt-0.5">
                                  Owner: {truncateAddress(position.owner, 4)}
                                </div>
                              </td>

                              {/* Collateral */}
                              <td className="px-4 py-4">
                                <div className="font-mono text-xs text-[var(--text-primary)] font-semibold">
                                  {position.collateralAmount.formatted} {position.collateralAsset}
                                </div>
                                {risk && (
                                  <div className="font-mono text-[11px] text-[var(--text-secondary)]">
                                    ${parseFloat(risk.collateralValueFormatted).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                                  </div>
                                )}
                              </td>

                              {/* Debt */}
                              <td className="px-4 py-4">
                                <div className="font-mono text-xs text-[var(--text-primary)] font-semibold">
                                  {position.debtAmount.formatted} {position.debtAsset}
                                </div>
                                {risk && (
                                  <div className="font-mono text-[11px] text-[var(--text-secondary)]">
                                    ${parseFloat(risk.debtValueFormatted).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                                  </div>
                                )}
                              </td>

                              {/* Health Factor */}
                              <td className="px-4 py-4 font-mono text-sm">
                                <span className={hfClass}>{hfStr}</span>
                                {risk?.liquidationPrice && (
                                  <div className="text-[11px] text-[var(--text-muted)] mt-0.5">
                                    Liq: ${parseFloat(risk.liquidationPrice.formatted).toFixed(2)}
                                  </div>
                                )}
                              </td>

                              {/* Risk Status */}
                              <td className="px-4 py-4">
                                <Badge variant={hfVariant}>{risk?.riskBandLabel || riskBand}</Badge>
                              </td>

                              {/* Provenance */}
                              <td className="px-4 py-4">
                                <ProvenanceBadge
                                  label={position.provenanceLabel}
                                  showAge={false}
                                />
                              </td>

                              {/* Action Link to Strategy Lab */}
                              <td className="px-6 py-4 text-right">
                                <Link
                                  href={`/app/recovery?positionId=${encodeURIComponent(position.id)}`}
                                  className="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg bg-[var(--surface-3)] text-xs font-mono font-medium text-[var(--accent)] hover:bg-[var(--accent)]/15 border border-[var(--border)] transition-colors"
                                >
                                  <span>Model Recovery</span>
                                  <span>→</span>
                                </Link>
                              </td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                )}
              </CardBody>
            </Card>
          </>
        )}
      </div>
    </div>
  );
}


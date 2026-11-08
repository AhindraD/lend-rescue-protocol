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

export function PositionsView() {
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

  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title="Positions"
        subtitle="Monitored Lending Obligations across Solana Protocol Markets"
        provenanceLabel={items[0]?.position.provenanceLabel || "Live"}
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
        {error && (
          <ErrorState
            title="Backend Communication Error"
            message={error.message}
            code={error.code}
            requestId={error.requestId}
            onRetry={fetchData}
          />
        )}

        {/* Filter and Search Bar */}
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-4">
          <input
            type="text"
            placeholder="Search position ID, wallet address, or asset..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="px-4 py-2 rounded-lg border border-[var(--border)] bg-[var(--surface-1)] text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] font-mono flex-1 max-w-md"
          />

          <div className="flex rounded-lg border border-[var(--border)] p-1 bg-[var(--surface-1)] text-xs font-mono">
            {["all", "critical", "warning", "safe"].map((tab) => (
              <button
                key={tab}
                onClick={() => setFilterRisk(tab)}
                className={`px-3 py-1.5 rounded-md transition-colors cursor-pointer capitalize ${
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

        {/* Positions Cards / Detailed Grid */}
        {loading && items.length === 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {Array.from({ length: 4 }).map((_, i) => (
              <SkeletonCard key={i} />
            ))}
          </div>
        ) : filteredPositions.length === 0 ? (
          <EmptyState
            title="No matching positions found"
            description="No positions found matching your search and filter criteria."
          />
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {filteredPositions.map(({ position, risk }) => {
              const hfStr = risk?.healthFactor || "—";
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

              const isMultiAsset =
                (position.collateralPositions && position.collateralPositions.length > 1) ||
                (position.debtPositions && position.debtPositions.length > 1);

              return (
                <Card key={position.id} className="hover:border-[var(--border)]/80 transition-all flex flex-col justify-between">
                  <CardHeader>
                    <div>
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-xs font-bold text-[var(--text-primary)]">
                          {truncateAddress(position.id, 8)}
                        </span>
                        {isMultiAsset && (
                          <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-[var(--surface-3)] text-[var(--text-muted)] border border-[var(--border)]">
                            Multi-Asset
                          </span>
                        )}
                      </div>
                      <CardDescription className="font-mono text-[11px] mt-0.5">
                        Owner: {truncateAddress(position.owner, 6)} · {position.protocolSource}
                      </CardDescription>
                    </div>
                    <Badge variant={badgeVariant}>{risk?.riskBandLabel || riskBand}</Badge>
                  </CardHeader>

                  <CardBody className="space-y-4">
                    {/* Collateral & Debt Snapshot */}
                    <div className="grid grid-cols-2 gap-4 p-3 rounded-lg bg-[var(--surface-2)]/60 border border-[var(--border)] font-mono">
                      <div>
                        <p className="text-[11px] uppercase text-[var(--text-muted)]">Collateral</p>
                        <p className="text-sm font-semibold text-[var(--text-primary)] mt-0.5">
                          {position.collateralAmount.formatted} {position.collateralAsset}
                        </p>
                        {risk && (
                          <p className="text-xs text-[var(--text-secondary)]">
                            ${parseFloat(risk.collateralValueFormatted).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                          </p>
                        )}
                      </div>

                      <div>
                        <p className="text-[11px] uppercase text-[var(--text-muted)]">Debt</p>
                        <p className="text-sm font-semibold text-[var(--text-primary)] mt-0.5">
                          {position.debtAmount.formatted} {position.debtAsset}
                        </p>
                        {risk && (
                          <p className="text-xs text-[var(--text-secondary)]">
                            ${parseFloat(risk.debtValueFormatted).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                          </p>
                        )}
                      </div>
                    </div>

                    {/* Financial Metrics */}
                    <div className="grid grid-cols-2 gap-3 text-xs font-mono">
                      <div className="flex justify-between py-1 border-b border-[var(--border)]/60">
                        <span className="text-[var(--text-muted)]">Health Factor</span>
                        <span className={hfClass}>{hfStr}</span>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[var(--border)]/60">
                        <span className="text-[var(--text-muted)]">Liquidation LT</span>
                        <span className="text-[var(--text-primary)]">
                          {(position.liquidationThresholdBps / 100).toFixed(1)}%
                        </span>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[var(--border)]/60">
                        <span className="text-[var(--text-muted)]">Liquidation Price</span>
                        <span className="text-[var(--risk-warning)]">
                          {risk?.liquidationPrice ? `$${parseFloat(risk.liquidationPrice.formatted).toFixed(2)}` : "—"}
                        </span>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[var(--border)]/60">
                        <span className="text-[var(--text-muted)]">Provenance</span>
                        <span className="text-[var(--text-secondary)]">{position.provenanceLabel}</span>
                      </div>
                    </div>

                    {/* Action Bar */}
                    <div className="pt-2 flex items-center justify-between">
                      <ProvenanceBadge
                        label={position.provenanceLabel}
                        showAge={false}
                      />
                      <Link
                        href={`/app/recovery?positionId=${encodeURIComponent(position.id)}`}
                        className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg bg-[var(--accent)] text-[var(--background)] font-mono text-xs font-bold hover:bg-[var(--accent)]/90 transition-colors shadow-sm"
                      >
                        <span>Model Recovery Strategy</span>
                        <span>→</span>
                      </Link>
                    </div>
                  </CardBody>
                </Card>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

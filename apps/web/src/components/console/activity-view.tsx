"use client";

import { useEffect, useState } from "react";
import { getOraclePrice, listPositions, getPositionRisk } from "@/lib/api/client";
import type {
  OraclePriceResponse,
  PositionResponse,
  RiskAssessmentResponse,
} from "@/types/api";
import { ConsoleHeader } from "./header";
import { Card, CardBody, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton, SkeletonCard } from "@/components/ui/skeleton";
import { ErrorState } from "@/components/ui/error-state";
import { ProvenanceBadge } from "./provenance-badge";
import { truncateAddress } from "@/lib/utils";

interface PositionRiskEvent {
  position: PositionResponse;
  risk?: RiskAssessmentResponse;
}

export function ActivityView() {
  const [oracles, setOracles] = useState<OraclePriceResponse[]>([]);
  const [positionsWithRisk, setPositionsWithRisk] = useState<PositionRiskEvent[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<{ message: string; code?: string } | null>(null);

  const fetchActivity = async () => {
    setLoading(true);
    setError(null);

    try {
      // 1. Fetch oracle price observations for SOL and USDC
      const oracleResults: OraclePriceResponse[] = [];
      for (const asset of ["SOL", "USDC"]) {
        try {
          const obs = await getOraclePrice(asset);
          oracleResults.push(obs);
        } catch {
          // Some feeds may be absent in synthetic environment
        }
      }
      setOracles(oracleResults);

      // 2. Fetch positions and their recent evaluations
      const posList = await listPositions();
      const events: PositionRiskEvent[] = await Promise.all(
        posList.map(async (pos) => {
          try {
            const risk = await getPositionRisk(pos.id);
            return { position: pos, risk };
          } catch {
            return { position: pos };
          }
        })
      );
      setPositionsWithRisk(events);
    } catch (err: unknown) {
      if (err instanceof Error) {
        setError({ message: err.message, code: (err as { code?: string }).code || "FETCH_FAILED" });
      } else {
        setError({ message: "Failed to load activity stream" });
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchActivity();
  }, []);

  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title="Activity &amp; Observations"
        subtitle="Real-Time Pyth Core Oracles, Market Health &amp; Position Risk Evaluations"
        provenanceLabel="Live Stream"
        actions={
          <Button
            size="sm"
            variant="secondary"
            onClick={fetchActivity}
            loading={loading}
          >
            ↻ Refresh Log
          </Button>
        }
      />

      <div className="flex-1 p-6 space-y-6 max-w-7xl w-full">
        {error && (
          <ErrorState
            title="Activity Feed Error"
            message={error.message}
            code={error.code}
            onRetry={fetchActivity}
          />
        )}

        {/* Oracle Observations Section */}
        <Card>
          <CardHeader>
            <div>
              <CardTitle>Pyth Core Price Observations</CardTitle>
              <CardDescription>
                Sub-second push oracle feeds providing authoritative price data for liquidation safety
              </CardDescription>
            </div>
            <span className="text-xs font-mono text-[var(--risk-safe)] flex items-center gap-1.5">
              <span className="h-2 w-2 rounded-full bg-[var(--risk-safe)] animate-pulse" />
              Push Receiver Active
            </span>
          </CardHeader>
          <CardBody>
            {loading && oracles.length === 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <SkeletonCard />
                <SkeletonCard />
              </div>
            ) : oracles.length === 0 ? (
              <p className="text-xs font-mono text-[var(--text-muted)] py-4">
                No active oracle feeds returned by backend.
              </p>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {oracles.map((oracle) => {
                  const confVal = parseFloat(oracle.price.confidence);
                  const isHealthy = !oracle.isStale && oracle.isValid;

                  return (
                    <div
                      key={oracle.assetId}
                      className="p-4 rounded-xl border border-[var(--border)] bg-[var(--surface-2)]/60 font-mono space-y-3"
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <span className="h-7 w-7 rounded-lg bg-[var(--surface-3)] border border-[var(--border)] flex items-center justify-center font-bold text-xs text-[var(--accent)]">
                            {oracle.assetId[0]}
                          </span>
                          <div>
                            <span className="text-sm font-bold text-[var(--text-primary)]">
                              {oracle.assetId} / USD
                            </span>
                            <span className="block text-[10px] text-[var(--text-muted)]">
                              Source: {oracle.dataSource}
                            </span>
                          </div>
                        </div>

                        <Badge variant={isHealthy ? "safe" : "warning"}>
                          {isHealthy ? "Fresh" : "Stale"}
                        </Badge>
                      </div>

                      <div className="flex items-baseline justify-between pt-1 border-t border-[var(--border)]/60">
                        <div>
                          <p className="text-[10px] uppercase text-[var(--text-muted)]">Verified Price</p>
                          <p className="text-2xl font-bold text-[var(--text-primary)]">
                            ${parseFloat(oracle.price.formatted).toFixed(4)}
                          </p>
                        </div>
                        <div className="text-right">
                          <p className="text-[10px] uppercase text-[var(--text-muted)]">Confidence Band</p>
                          <p className="text-xs text-[var(--text-secondary)]">
                            ±${confVal > 0 ? (confVal * Math.pow(10, oracle.price.exponent)).toFixed(4) : "0.0000"}
                          </p>
                        </div>
                      </div>

                      <div className="grid grid-cols-3 gap-2 pt-2 text-[10px] text-[var(--text-muted)] border-t border-[var(--border)]/60">
                        <div>
                          <span>Slot: </span>
                          <span className="text-[var(--text-secondary)] font-semibold">{oracle.price.slot}</span>
                        </div>
                        <div>
                          <span>Age: </span>
                          <span className="text-[var(--text-secondary)] font-semibold">{oracle.ageSecs}s</span>
                        </div>
                        <div>
                          <span>Exponent: </span>
                          <span className="text-[var(--text-secondary)] font-semibold">{oracle.price.exponent}</span>
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </CardBody>
        </Card>

        {/* Position Evaluation Timeline */}
        <Card>
          <CardHeader>
            <div>
              <CardTitle>Recent Risk Engine Observations</CardTitle>
              <CardDescription>
                Chronological risk assessments generated by the Rust engine
              </CardDescription>
            </div>
          </CardHeader>
          <CardBody className="p-0">
            {loading && positionsWithRisk.length === 0 ? (
              <div className="p-6 space-y-3">
                <Skeleton className="h-8 w-full" />
                <Skeleton className="h-8 w-full" />
                <Skeleton className="h-8 w-full" />
              </div>
            ) : positionsWithRisk.length === 0 ? (
              <div className="p-8 text-center text-xs font-mono text-[var(--text-muted)]">
                No evaluated position observations logged.
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm font-mono">
                  <thead className="bg-[var(--surface-2)]/60 border-b border-[var(--border)] text-xs uppercase text-[var(--text-muted)]">
                    <tr>
                      <th className="px-6 py-3">Obligation ID</th>
                      <th className="px-4 py-3">Asset Pair</th>
                      <th className="px-4 py-3">Health Factor</th>
                      <th className="px-4 py-3">Risk Band</th>
                      <th className="px-4 py-3">Provenance</th>
                      <th className="px-6 py-3 text-right">Observation Time</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[var(--border)] text-xs">
                    {positionsWithRisk.map(({ position, risk }) => {
                      const hf = risk?.healthFactor || "—";
                      const band = risk?.riskBand || "Unknown";
                      const evalTime = risk?.evaluatedAt
                        ? new Date(risk.evaluatedAt * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" })
                        : "Recent";

                      return (
                        <tr key={position.id} className="hover:bg-[var(--surface-2)]/40 transition-colors">
                          <td className="px-6 py-3.5 font-bold text-[var(--text-primary)]">
                            {truncateAddress(position.id, 6)}
                          </td>
                          <td className="px-4 py-3.5 text-[var(--text-secondary)]">
                            {position.collateralAsset} / {position.debtAsset}
                          </td>
                          <td className="px-4 py-3.5 font-bold">
                            <span className={band.toLowerCase() === "critical" ? "text-[var(--risk-critical)]" : band.toLowerCase().includes("warn") ? "text-[var(--risk-warning)]" : "text-[var(--risk-safe)]"}>
                              {hf}
                            </span>
                          </td>
                          <td className="px-4 py-3.5">
                            <Badge
                              variant={
                                band.toLowerCase() === "critical"
                                  ? "critical"
                                  : band.toLowerCase().includes("warn")
                                  ? "warning"
                                  : "safe"
                              }
                            >
                              {risk?.riskBandLabel || band}
                            </Badge>
                          </td>
                          <td className="px-4 py-3.5">
                            <ProvenanceBadge label={position.provenanceLabel} showAge={false} />
                          </td>
                          <td className="px-6 py-3.5 text-right text-[var(--text-muted)]">
                            {evalTime}
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
      </div>
    </div>
  );
}

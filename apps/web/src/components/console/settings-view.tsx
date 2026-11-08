"use client";

import { useEffect, useState } from "react";
import { getHealth, getReady } from "@/lib/api/client";
import type { HealthResponse, ReadyResponse } from "@/types/api";
import { ConsoleHeader } from "./header";
import { Card, CardBody, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";

export function SettingsView() {
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [ready, setReady] = useState<ReadyResponse | null>(null);
  const [checking, setChecking] = useState<boolean>(true);
  const [connectionError, setConnectionError] = useState<string | null>(null);

  const checkConnectivity = async () => {
    setChecking(true);
    setConnectionError(null);
    try {
      const [h, r] = await Promise.all([getHealth(), getReady()]);
      setHealth(h);
      setReady(r);
    } catch (err: unknown) {
      if (err instanceof Error) {
        setConnectionError(err.message);
      } else {
        setConnectionError("Failed to reach Axum backend API");
      }
    } finally {
      setChecking(false);
    }
  };

  useEffect(() => {
    checkConnectivity();
  }, []);

  const apiUrl =
    process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, "") ??
    "http://localhost:8080";

  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title="Settings &amp; Environment Diagnostics"
        subtitle="Backend Connectivity, Protocol Addresses &amp; Security Invariant Policies"
        provenanceLabel={ready?.ready ? "API Connected" : "Checking"}
        actions={
          <Button
            size="sm"
            variant="secondary"
            onClick={checkConnectivity}
            loading={checking}
          >
            ↻ Test Connection
          </Button>
        }
      />

      <div className="flex-1 p-6 space-y-6 max-w-4xl w-full">
        {/* Backend Connectivity Status */}
        <Card>
          <CardHeader>
            <div>
              <CardTitle>Axum Backend Connectivity</CardTitle>
              <CardDescription>
                Live status check of the Rust calculation service and SQLite database
              </CardDescription>
            </div>
            {checking ? (
              <Badge variant="muted">Checking...</Badge>
            ) : connectionError ? (
              <Badge variant="critical">Disconnected</Badge>
            ) : (
              <Badge variant="safe">Healthy &amp; Ready</Badge>
            )}
          </CardHeader>
          <CardBody className="space-y-4">
            <div>
              <label className="text-xs font-mono uppercase text-[var(--text-muted)] block mb-1">
                Configured API Base URL
              </label>
              <div className="flex items-center justify-between p-3 rounded-lg bg-[var(--surface-2)] border border-[var(--border)] font-mono text-xs">
                <span className="text-[var(--text-primary)] font-semibold">{apiUrl}</span>
                <a
                  href={`${apiUrl}/api/v1/openapi.json`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-[var(--accent)] hover:underline"
                >
                  View OpenAPI Spec ↗
                </a>
              </div>
            </div>

            {checking ? (
              <div className="space-y-2 pt-2">
                <Skeleton className="h-4 w-1/2" />
                <Skeleton className="h-4 w-1/3" />
              </div>
            ) : connectionError ? (
              <div className="p-3 rounded-lg bg-[var(--risk-critical)]/10 border border-[var(--risk-critical)]/30 text-xs font-mono text-[var(--risk-critical)]">
                Error: {connectionError}
              </div>
            ) : (
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 pt-2 text-xs font-mono">
                <div className="p-3 rounded-lg bg-[var(--surface-2)] border border-[var(--border)]">
                  <span className="text-[10px] text-[var(--text-muted)] block uppercase">Service</span>
                  <span className="font-semibold text-[var(--text-primary)]">{health?.service || "lend-rescue-api"}</span>
                </div>
                <div className="p-3 rounded-lg bg-[var(--surface-2)] border border-[var(--border)]">
                  <span className="text-[10px] text-[var(--text-muted)] block uppercase">Version</span>
                  <span className="font-semibold text-[var(--text-primary)]">{health?.version || "0.1.0"}</span>
                </div>
                <div className="p-3 rounded-lg bg-[var(--surface-2)] border border-[var(--border)]">
                  <span className="text-[10px] text-[var(--text-muted)] block uppercase">Database</span>
                  <span className="font-semibold text-[var(--risk-safe)]">{ready?.database || "Connected"}</span>
                </div>
                <div className="p-3 rounded-lg bg-[var(--surface-2)] border border-[var(--border)]">
                  <span className="text-[10px] text-[var(--text-muted)] block uppercase">Mode</span>
                  <span className="font-semibold text-[var(--accent)]">
                    {ready?.demoMode ? "Demo Mode" : "Production / Live"}
                  </span>
                </div>
              </div>
            )}
          </CardBody>
        </Card>

        {/* Protocol Addresses & Verified Identifiers */}
        <Card>
          <CardHeader>
            <div>
              <CardTitle>On-Chain Protocol Identifiers</CardTitle>
              <CardDescription>Verified Solana program and feed addresses used by protocol adapters</CardDescription>
            </div>
          </CardHeader>
          <CardBody>
            <div className="space-y-3 font-mono text-xs">
              <div className="flex flex-col sm:flex-row sm:items-center justify-between p-2.5 rounded-lg bg-[var(--surface-2)] border border-[var(--border)] gap-2">
                <div>
                  <span className="text-[var(--text-primary)] font-semibold block">Kamino Main Lending Market</span>
                  <span className="text-[10px] text-[var(--text-muted)]">Official Kamino Lending program market address</span>
                </div>
                <span className="text-[var(--accent)] text-[11px] select-all break-all">
                  7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF
                </span>
              </div>

              <div className="flex flex-col sm:flex-row sm:items-center justify-between p-2.5 rounded-lg bg-[var(--surface-2)] border border-[var(--border)] gap-2">
                <div>
                  <span className="text-[var(--text-primary)] font-semibold block">Pyth Push Oracle Receiver</span>
                  <span className="text-[10px] text-[var(--text-muted)]">Push oracle program ID decoding pull feeds</span>
                </div>
                <span className="text-[var(--risk-safe)] text-[11px] select-all break-all">
                  rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ
                </span>
              </div>

              <div className="flex flex-col sm:flex-row sm:items-center justify-between p-2.5 rounded-lg bg-[var(--surface-2)] border border-[var(--border)] gap-2">
                <div>
                  <span className="text-[var(--text-primary)] font-semibold block">SOL / USD Feed Account</span>
                  <span className="text-[10px] text-[var(--text-muted)]">Pyth price feed account (Shard 0)</span>
                </div>
                <span className="text-[var(--text-secondary)] text-[11px] select-all break-all">
                  7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE
                </span>
              </div>

              <div className="flex flex-col sm:flex-row sm:items-center justify-between p-2.5 rounded-lg bg-[var(--surface-2)] border border-[var(--border)] gap-2">
                <div>
                  <span className="text-[var(--text-primary)] font-semibold block">USDC / USD Feed Account</span>
                  <span className="text-[10px] text-[var(--text-muted)]">Pyth price feed account (Shard 0)</span>
                </div>
                <span className="text-[var(--text-secondary)] text-[11px] select-all break-all">
                  Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX
                </span>
              </div>

              <div className="flex flex-col sm:flex-row sm:items-center justify-between p-2.5 rounded-lg bg-[var(--surface-2)] border border-[var(--border)] gap-2">
                <div>
                  <span className="text-[var(--text-primary)] font-semibold block">Jupiter Swap API V2</span>
                  <span className="text-[10px] text-[var(--text-muted)]">DEX quote router endpoint (Quote-Only)</span>
                </div>
                <span className="text-[var(--risk-warning)] text-[11px]">
                  Quote-Only Routing · No Execution
                </span>
              </div>
            </div>
          </CardBody>
        </Card>

        {/* Non-Custodial Safety Policies */}
        <Card>
          <CardHeader>
            <div>
              <CardTitle>Non-Custodial Runtime Boundaries</CardTitle>
              <CardDescription>Verified engineering constraints and safety guarantees</CardDescription>
            </div>
          </CardHeader>
          <CardBody className="space-y-3 font-mono text-xs">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {[
                { title: "Key Management", status: "Zero Custody", detail: "Private keys or seed phrases are never accepted or stored." },
                { title: "Transaction Construction", status: "Disabled", detail: "The app cannot sign or broadcast transactions to Solana." },
                { title: "Risk Calculations", status: "Server Enforced", detail: "Pure Rust engines compute exact deterministic metrics." },
                { title: "Multi-Asset Invariant", status: "Fail-Closed", detail: "Complex multi-asset obligations return explicit HTTP 422 errors." },
                { title: "Swap Quotes", status: "Indicative Only", detail: "Jupiter DEX quotes are strictly advisory snapshots." },
                { title: "Provenance Tracking", status: "Full Auditability", detail: "Every observation carries source and staleness flags." },
              ].map((item) => (
                <div key={item.title} className="p-3 rounded-lg bg-[var(--surface-2)]/60 border border-[var(--border)]">
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-[var(--text-primary)] font-semibold">{item.title}</span>
                    <span className="text-[var(--risk-safe)] font-bold text-[11px]">{item.status}</span>
                  </div>
                  <p className="text-[11px] text-[var(--text-muted)] leading-relaxed">{item.detail}</p>
                </div>
              ))}
            </div>
          </CardBody>
        </Card>
      </div>
    </div>
  );
}

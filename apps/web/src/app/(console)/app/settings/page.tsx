import type { Metadata } from "next";
import { ConsoleHeader } from "@/components/console/header";
import { Card, CardBody, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";

export const metadata: Metadata = {
  title: "Settings",
  description: "API environment and application configuration",
};

export default function SettingsPage() {
  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title="Settings"
        subtitle="Environment endpoints and protocol integration settings"
      />
      <div className="flex-1 p-6 max-w-3xl space-y-6">
        <Card>
          <CardHeader>
            <div>
              <CardTitle>API Configuration</CardTitle>
              <CardDescription>
                Backend Axum service connection
              </CardDescription>
            </div>
          </CardHeader>
          <CardBody>
            <div className="space-y-4">
              <div>
                <label className="text-xs font-mono uppercase text-[var(--text-muted)] block mb-1">
                  API URL
                </label>
                <div className="p-3 rounded-lg bg-[var(--surface-2)] border border-[var(--border)] font-mono text-sm text-[var(--text-primary)]">
                  {process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080"}
                </div>
              </div>
              <p className="text-xs text-[var(--text-muted)]">
                The frontend communicates strictly with this endpoint. No client-side RPC credentials or secret keys are configured.
              </p>
            </div>
          </CardBody>
        </Card>

        <Card>
          <CardHeader>
            <div>
              <CardTitle>Safety Boundaries</CardTitle>
              <CardDescription>Non-custodial runtime policies</CardDescription>
            </div>
          </CardHeader>
          <CardBody>
            <div className="space-y-2 text-xs text-[var(--text-secondary)] font-mono">
              <div className="flex items-center gap-2">
                <span className="text-[var(--risk-safe)]">✓</span>
                <span>Transaction Signing: Disabled</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-[var(--risk-safe)]">✓</span>
                <span>Client Financial Computation: Disabled (Engine Enforced)</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-[var(--risk-safe)]">✓</span>
                <span>Direct Protocol Execution: Disabled (Advisory Only)</span>
              </div>
            </div>
          </CardBody>
        </Card>
      </div>
    </div>
  );
}

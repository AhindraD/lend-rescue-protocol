import type { Metadata } from "next";
import { ConsoleHeader } from "@/components/console/header";
import { Card, CardBody, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Dashboard",
  description: "Lending position overview and real-time risk status",
};

export default function DashboardPage() {
  return (
    <div className="h-full flex flex-col">
      <ConsoleHeader
        title="Dashboard"
        subtitle="Lending risk intelligence and active position overview"
      />
      <div className="flex-1 p-6 space-y-6 max-w-7xl">
        {/* Metric Overview Cards */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {[
            { label: "Total Collateral", hint: "Across monitored reserves" },
            { label: "Total Borrowed", hint: "Active obligations" },
            { label: "Critical Risk Positions", hint: "HF < 1.10" },
            { label: "Active Positions", hint: "Kamino Main Market" },
          ].map((item) => (
            <Card key={item.label}>
              <CardBody>
                <p className="text-xs font-mono uppercase tracking-wider text-[var(--text-muted)] mb-2">
                  {item.label}
                </p>
                <div className="flex items-baseline gap-2 mb-1">
                  <Skeleton className="h-8 w-28 rounded" />
                </div>
                <p className="text-xs text-[var(--text-muted)]">{item.hint}</p>
              </CardBody>
            </Card>
          ))}
        </div>

        {/* Quick Actions & Lab Link */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <Card className="lg:col-span-2">
            <CardHeader>
              <div>
                <CardTitle>Monitored Obligations</CardTitle>
                <CardDescription>
                  Positions tracked by the risk engine
                </CardDescription>
              </div>
              <Link
                href="/app/positions"
                className="text-xs font-mono text-[var(--accent)] hover:underline"
              >
                View all →
              </Link>
            </CardHeader>
            <CardBody>
              <div className="space-y-3">
                {Array.from({ length: 3 }).map((_, i) => (
                  <div
                    key={i}
                    className="flex items-center justify-between p-3 rounded-lg border border-[var(--border)] bg-[var(--surface-2)]/50"
                  >
                    <div className="flex items-center gap-3">
                      <Skeleton className="h-9 w-9 rounded-lg" />
                      <div className="space-y-1.5">
                        <Skeleton className="h-4 w-32" />
                        <Skeleton className="h-3 w-20" />
                      </div>
                    </div>
                    <Skeleton className="h-6 w-20 rounded" />
                  </div>
                ))}
              </div>
            </CardBody>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Recovery Strategy Lab</CardTitle>
              <CardDescription>Deterministic solver</CardDescription>
            </CardHeader>
            <CardBody>
              <p className="text-sm text-[var(--text-secondary)] mb-4 leading-relaxed">
                Run the recovery engine to model candidate plans for high-risk positions
                and view indicative Jupiter swap routing.
              </p>
              <Link
                href="/app/recovery"
                className="inline-flex w-full items-center justify-center px-4 py-2 rounded-lg bg-[var(--surface-2)] border border-[var(--border)] text-sm font-medium text-[var(--text-primary)] hover:bg-[var(--surface-3)] transition-colors"
              >
                Open Strategy Lab →
              </Link>
            </CardBody>
          </Card>
        </div>
      </div>
    </div>
  );
}

import { cn, timeAgo } from "@/lib/utils";
import type { Provenance, ProvenanceInfo } from "@/types/api";

const provenanceConfig: Record<
  Provenance,
  { label: string; dotClass: string; textClass: string }
> = {
  Live: {
    label: "Live",
    dotClass: "bg-[var(--risk-safe)] animate-pulse",
    textClass: "text-[var(--risk-safe)]",
  },
  SurfpoolSimulated: {
    label: "Surfpool Simulated",
    dotClass: "bg-[var(--risk-warning)]",
    textClass: "text-[var(--risk-warning)]",
  },
  MockOracle: {
    label: "Mock Oracle",
    dotClass: "bg-[var(--text-muted)]",
    textClass: "text-[var(--text-muted)]",
  },
  Cached: {
    label: "Cached",
    dotClass: "bg-[var(--accent)]",
    textClass: "text-[var(--accent)]",
  },
  Demo: {
    label: "Demo",
    dotClass: "bg-[var(--text-muted)]",
    textClass: "text-[var(--text-muted)]",
  },
};

interface ProvenanceBadgeProps {
  provenance: ProvenanceInfo;
  className?: string;
  showAge?: boolean;
}

export function ProvenanceBadge({
  provenance,
  className,
  showAge = true,
}: ProvenanceBadgeProps) {
  const config =
    provenanceConfig[provenance.source] ?? provenanceConfig.Demo;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-mono border border-[var(--border)] bg-[var(--surface-2)]/80",
        config.textClass,
        className
      )}
      title={`Data source: ${provenance.source}${
        provenance.timestamp_ms
          ? ` · ${new Date(provenance.timestamp_ms).toISOString()}`
          : ""
      }`}
    >
      <span
        className={cn("h-1.5 w-1.5 rounded-full shrink-0", config.dotClass)}
      />
      <span>{config.label}</span>
      {showAge && provenance.timestamp_ms > 0 && (
        <span className="text-[var(--text-muted)]">
          · {timeAgo(provenance.timestamp_ms)}
        </span>
      )}
    </span>
  );
}


import { cn } from "@/lib/utils";

interface ProvenanceBadgeProps {
  label: string;
  timestamp?: number;
  className?: string;
  showAge?: boolean;
}

export function ProvenanceBadge({
  label,
  timestamp,
  className,
  showAge = true,
}: ProvenanceBadgeProps) {
  const norm = (label || "").toLowerCase();

  let dotColor = "bg-[var(--text-muted)]";
  let textColor = "text-[var(--text-muted)]";
  let displayLabel = label || "Observed";

  if (norm.includes("live") || norm.includes("pyth core")) {
    dotColor = "bg-[var(--risk-safe)] animate-pulse";
    textColor = "text-[var(--risk-safe)]";
    displayLabel = "Live";
  } else if (norm.includes("surfpool") || norm.includes("simulated")) {
    dotColor = "bg-[var(--risk-warning)]";
    textColor = "text-[var(--risk-warning)]";
    displayLabel = "Surfpool Simulated";
  } else if (norm.includes("mock")) {
    dotColor = "bg-[var(--text-muted)]";
    textColor = "text-[var(--text-muted)]";
    displayLabel = "Mock Oracle";
  } else if (norm.includes("cached") || norm.includes("database")) {
    dotColor = "bg-[var(--accent)]";
    textColor = "text-[var(--accent)]";
    displayLabel = "Cached";
  }

  const ageText =
    timestamp && timestamp > 0
      ? ` · ${new Date(timestamp * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`
      : "";

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-mono border border-[var(--border)] bg-[var(--surface-2)]",
        textColor,
        className
      )}
      title={`Data Provenance: ${label}`}
    >
      <span className={cn("h-1.5 w-1.5 rounded-full shrink-0", dotColor)} />
      <span>{displayLabel}</span>
      {showAge && ageText && (
        <span className="text-[var(--text-muted)]">{ageText}</span>
      )}
    </span>
  );
}

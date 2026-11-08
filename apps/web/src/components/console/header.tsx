import type { ProvenanceInfo } from "@/types/api";
import { ProvenanceBadge } from "./provenance-badge";

interface ConsoleHeaderProps {
  title: string;
  subtitle?: string;
  provenance?: ProvenanceInfo;
  actions?: React.ReactNode;
}

export function ConsoleHeader({
  title,
  subtitle,
  provenance,
  actions,
}: ConsoleHeaderProps) {
  return (
    <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 px-6 py-4 border-b border-[var(--border)] bg-[var(--surface-1)]">
      <div>
        <h1 className="text-lg font-semibold text-[var(--text-primary)] tracking-tight">
          {title}
        </h1>
        {subtitle && (
          <p className="text-xs text-[var(--text-muted)] mt-0.5">
            {subtitle}
          </p>
        )}
      </div>
      <div className="flex items-center gap-3">
        {provenance && (
          <ProvenanceBadge provenance={provenance} />
        )}
        {actions}
      </div>
    </div>
  );
}

import { cn } from "@/lib/utils";
import type { RiskLevel } from "@/types/api";

type BadgeVariant =
  | "default"
  | "accent"
  | "safe"
  | "warning"
  | "critical"
  | "muted";

interface BadgeProps {
  children: React.ReactNode;
  variant?: BadgeVariant;
  className?: string;
}

const variantClasses: Record<BadgeVariant, string> = {
  default:
    "bg-[var(--surface-3)] text-[var(--text-secondary)] border-[var(--border)]",
  accent:
    "bg-[var(--accent)]/10 text-[var(--accent)] border-[var(--accent)]/30",
  safe: "bg-[var(--risk-safe)]/10 text-[var(--risk-safe)] border-[var(--risk-safe)]/30",
  warning:
    "bg-[var(--risk-warning)]/10 text-[var(--risk-warning)] border-[var(--risk-warning)]/30",
  critical:
    "bg-[var(--risk-critical)]/10 text-[var(--risk-critical)] border-[var(--risk-critical)]/30",
  muted:
    "bg-[var(--surface-2)] text-[var(--text-muted)] border-[var(--border)]",
};

export function Badge({
  children,
  variant = "default",
  className,
}: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium border font-mono",
        variantClasses[variant],
        className
      )}
    >
      {children}
    </span>
  );
}

export function RiskBadge({ level }: { level: RiskLevel }) {
  const variantMap: Record<RiskLevel, BadgeVariant> = {
    Safe: "safe",
    Warning: "warning",
    Critical: "critical",
    Unknown: "muted",
  };
  return <Badge variant={variantMap[level] ?? "muted"}>{level}</Badge>;
}


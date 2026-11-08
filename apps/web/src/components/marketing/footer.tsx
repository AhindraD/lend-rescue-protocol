import Link from "next/link";

export function MarketingFooter() {
  return (
    <footer className="border-t border-[var(--border)] bg-[var(--surface-1)] mt-auto">
      <div className="mx-auto max-w-7xl px-6 py-12 flex flex-col md:flex-row items-center justify-between gap-6 text-sm text-[var(--text-muted)]">
        <div className="flex flex-col sm:flex-row items-center gap-3 text-center sm:text-left">
          <div className="flex items-center gap-2">
            <span className="text-[var(--accent)] font-mono">◈</span>
            <span className="font-semibold text-[var(--text-primary)]">LendRescue</span>
          </div>
          <span className="hidden sm:inline text-[var(--border)]">•</span>
          <span>Advisory &amp; non-custodial risk engine. No funds held. No transactions executed.</span>
        </div>
        <div className="flex items-center gap-6">
          <Link
            href="/#security"
            className="hover:text-[var(--text-secondary)] transition-colors"
          >
            Security Model
          </Link>
          <span className="text-xs font-mono px-2 py-0.5 rounded border border-[var(--border)] bg-[var(--surface-2)]">
            Solana
          </span>
        </div>
      </div>
    </footer>
  );
}


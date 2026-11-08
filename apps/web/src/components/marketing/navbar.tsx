import Link from "next/link";

export function MarketingNavbar() {
  return (
    <header className="sticky top-0 z-50 border-b border-[var(--border)] bg-[var(--background)]/85 backdrop-blur-md">
      <div className="mx-auto max-w-7xl px-6 flex h-16 items-center justify-between">
        <Link
          href="/"
          className="flex items-center gap-2 font-semibold text-[var(--text-primary)] hover:opacity-90"
        >
          <span className="text-[var(--accent)] font-mono text-xl">◈</span>
          <span className="tracking-tight text-base font-bold">LendRescue</span>
        </Link>
        <nav className="flex items-center gap-6 text-sm text-[var(--text-secondary)]">
          <Link
            href="/#how-it-works"
            className="hidden sm:inline-block hover:text-[var(--text-primary)] transition-colors"
          >
            How it works
          </Link>
          <Link
            href="/#protocols"
            className="hidden sm:inline-block hover:text-[var(--text-primary)] transition-colors"
          >
            Protocols
          </Link>
          <Link
            href="/#security"
            className="hidden sm:inline-block hover:text-[var(--text-primary)] transition-colors"
          >
            Security
          </Link>
          <Link
            href="/app/dashboard"
            className="px-4 py-2 rounded-lg bg-[var(--accent)] text-[var(--background)] font-semibold text-sm hover:bg-[var(--accent)]/90 transition-colors shadow-sm"
          >
            Open Console →
          </Link>
        </nav>
      </div>
    </header>
  );
}


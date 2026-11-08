import Link from "next/link";

export default function NotFound() {
  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-[var(--background)] text-[var(--text-primary)] px-4">
      <span className="text-[var(--accent)] font-mono text-4xl mb-4">◈</span>
      <h1 className="text-2xl font-bold mb-2">Page not found</h1>
      <p className="text-[var(--text-secondary)] mb-6 text-sm">
        The page you are looking for does not exist or has moved.
      </p>
      <Link
        href="/"
        className="px-4 py-2 rounded-md bg-[var(--surface-2)] text-[var(--text-primary)] border border-[var(--border)] hover:bg-[var(--surface-3)] transition-colors text-sm"
      >
        Return home
      </Link>
    </div>
  );
}

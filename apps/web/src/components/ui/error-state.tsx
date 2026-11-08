"use client";

import { cn } from "@/lib/utils";

interface ErrorStateProps {
  title?: string;
  message: string;
  code?: string;
  requestId?: string;
  onRetry?: () => void;
  className?: string;
}

export function ErrorState({
  title = "Something went wrong",
  message,
  code,
  requestId,
  onRetry,
  className,
}: ErrorStateProps) {
  return (
    <div
      className={cn(
        "rounded-xl border border-[var(--risk-critical)]/25 bg-[var(--risk-critical)]/5 p-6",
        className
      )}
      role="alert"
    >
      <div className="flex items-start gap-3">
        <svg
          className="h-5 w-5 text-[var(--risk-critical)] mt-0.5 shrink-0"
          viewBox="0 0 20 20"
          fill="currentColor"
          aria-hidden="true"
        >
          <path
            fillRule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z"
            clipRule="evenodd"
          />
        </svg>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-semibold text-[var(--risk-critical)]">
            {title}
          </p>
          <p className="mt-1 text-sm text-[var(--text-secondary)]">
            {message}
          </p>
          {(code || requestId) && (
            <p className="mt-2 text-xs font-mono text-[var(--text-muted)] flex flex-wrap gap-4">
              {code && <span>Code: {code}</span>}
              {requestId && <span>Request ID: {requestId}</span>}
            </p>
          )}
          {onRetry && (
            <button
              onClick={onRetry}
              className="mt-3 text-sm text-[var(--accent)] hover:underline inline-flex items-center gap-1 cursor-pointer"
            >
              Try again →
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

/**
 * API contract constants and endpoint definitions matching Axum backend router.
 */

export const API_ENDPOINTS = {
  HEALTH: "/health",
  READY: "/ready",
  MARKETS: "/markets",
  POSITIONS: "/positions",
  POSITION_BY_ID: (id: string) => `/positions/${encodeURIComponent(id)}`,
  POSITION_RISK: (id: string) => `/positions/${encodeURIComponent(id)}/risk`,
  ORACLE_PRICE: (assetId: string) => `/oracles/${encodeURIComponent(assetId)}`,
  RECOVERY_QUOTE: "/recovery/quote",
  RECOVERY_OPTIMIZE: "/recovery/optimize",
  RECOVERY_BY_ID: (id: string) => `/recovery/${encodeURIComponent(id)}`,
  OPENAPI_SPEC: "/openapi.json",
} as const;

export * from "@/types/api";


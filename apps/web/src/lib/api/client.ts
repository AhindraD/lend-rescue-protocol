import {
  API_ENDPOINTS,
  type HealthResponse,
  type ReadyResponse,
  type ListMarketsResponse,
  type ListPositionsResponse,
  type Position,
  type RiskAssessment,
  type OraclePrice,
  type RecoveryQuoteRequest,
  type RecoveryQuoteResponse,
  type OptimizeRecoveryRequest,
  type RecoveryRecommendation,
  type ApiError,
} from "./contracts";

const API_BASE =
  process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, "") ??
  "http://localhost:8080";

export class ApiResponseError extends Error {
  constructor(
    public readonly status: number,
    public readonly body: ApiError,
    public readonly requestId?: string
  ) {
    super(body.error || `HTTP ${status}`);
    this.name = "ApiResponseError";
  }
}

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const url = `${API_BASE}/api/v1${path}`;
  const res = await fetch(url, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
      ...(init?.headers ?? {}),
    },
  });

  if (!res.ok) {
    let body: ApiError;
    try {
      body = (await res.json()) as ApiError;
    } catch {
      body = {
        error: res.statusText || `Request failed with status ${res.status}`,
        code: `HTTP_${res.status}`,
      };
    }
    const requestId = res.headers.get("x-request-id") ?? body.request_id ?? undefined;
    throw new ApiResponseError(res.status, body, requestId);
  }

  return res.json() as Promise<T>;
}

// ─── Health & Status ──────────────────────────────────────────────────────────

export const getHealth = (): Promise<HealthResponse> =>
  apiFetch<HealthResponse>(API_ENDPOINTS.HEALTH);

export const getReady = (): Promise<ReadyResponse> =>
  apiFetch<ReadyResponse>(API_ENDPOINTS.READY);

// ─── Markets ──────────────────────────────────────────────────────────────────

export const listMarkets = (): Promise<ListMarketsResponse> =>
  apiFetch<ListMarketsResponse>(API_ENDPOINTS.MARKETS);

// ─── Positions ────────────────────────────────────────────────────────────────

export const listPositions = (): Promise<ListPositionsResponse> =>
  apiFetch<ListPositionsResponse>(API_ENDPOINTS.POSITIONS);

export const getPosition = (positionId: string): Promise<Position> =>
  apiFetch<Position>(API_ENDPOINTS.POSITION_BY_ID(positionId));

export const getPositionRisk = (positionId: string): Promise<RiskAssessment> =>
  apiFetch<RiskAssessment>(API_ENDPOINTS.POSITION_RISK(positionId));

// ─── Oracles ──────────────────────────────────────────────────────────────────

export const getOraclePrice = (assetId: string): Promise<OraclePrice> =>
  apiFetch<OraclePrice>(API_ENDPOINTS.ORACLE_PRICE(assetId));

// ─── Recovery Strategy ────────────────────────────────────────────────────────

export const quoteRecovery = (
  body: RecoveryQuoteRequest
): Promise<RecoveryQuoteResponse> =>
  apiFetch<RecoveryQuoteResponse>(API_ENDPOINTS.RECOVERY_QUOTE, {
    method: "POST",
    body: JSON.stringify(body),
  });

export const optimizeRecovery = (
  body: OptimizeRecoveryRequest
): Promise<RecoveryRecommendation> =>
  apiFetch<RecoveryRecommendation>(API_ENDPOINTS.RECOVERY_OPTIMIZE, {
    method: "POST",
    body: JSON.stringify(body),
  });

export const getRecoveryRecommendation = (
  recoveryId: string
): Promise<RecoveryRecommendation> =>
  apiFetch<RecoveryRecommendation>(API_ENDPOINTS.RECOVERY_BY_ID(recoveryId));

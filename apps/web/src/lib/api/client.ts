import {
  API_ENDPOINTS,
  type HealthResponse,
  type ReadyResponse,
  type MarketResponse,
  type PositionResponse,
  type PositionFilterQuery,
  type RiskAssessmentResponse,
  type PriceDto,
  type RecoveryQuoteRequest,
  type RecoveryPlanDto,
  type RecoveryOptimizeRequest,
  type RecoveryRecommendationResponse,
  type ErrorResponse,
} from "./contracts";

const API_BASE =
  process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, "") ??
  "http://localhost:8080";

export class ApiResponseError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    public readonly message: string,
    public readonly requestId?: string
  ) {
    super(message || `HTTP ${status}`);
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
    let code = `HTTP_${res.status}`;
    let message = res.statusText || `Request failed with status ${res.status}`;
    let requestId = res.headers.get("x-request-id") ?? undefined;

    try {
      const errJson = (await res.json()) as ErrorResponse;
      if (errJson && errJson.error) {
        code = errJson.error.code || code;
        message = errJson.error.message || message;
        requestId = errJson.error.requestId || requestId;
      }
    } catch {
      // Body not JSON
    }

    throw new ApiResponseError(res.status, code, message, requestId);
  }

  return res.json() as Promise<T>;
}

// ─── Health & Status ──────────────────────────────────────────────────────────

export const getHealth = (): Promise<HealthResponse> =>
  apiFetch<HealthResponse>(API_ENDPOINTS.HEALTH);

export const getReady = (): Promise<ReadyResponse> =>
  apiFetch<ReadyResponse>(API_ENDPOINTS.READY);

// ─── Markets ──────────────────────────────────────────────────────────────────

export const listMarkets = (): Promise<MarketResponse[]> =>
  apiFetch<MarketResponse[]>(API_ENDPOINTS.MARKETS);

// ─── Positions ────────────────────────────────────────────────────────────────

export const listPositions = (
  query?: PositionFilterQuery
): Promise<PositionResponse[]> => {
  const params = new URLSearchParams();
  if (query?.owner) params.append("owner", query.owner);
  if (query?.protocol) params.append("protocol", query.protocol);
  if (query?.network) params.append("network", query.network);
  if (query?.riskBand) params.append("riskBand", query.riskBand);

  const qs = params.toString();
  const path = qs ? `${API_ENDPOINTS.POSITIONS}?${qs}` : API_ENDPOINTS.POSITIONS;
  return apiFetch<PositionResponse[]>(path);
};

export const getPosition = (positionId: string): Promise<PositionResponse> =>
  apiFetch<PositionResponse>(API_ENDPOINTS.POSITION_BY_ID(positionId));

export const getPositionRisk = (
  positionId: string
): Promise<RiskAssessmentResponse> =>
  apiFetch<RiskAssessmentResponse>(API_ENDPOINTS.POSITION_RISK(positionId));

// ─── Oracles ──────────────────────────────────────────────────────────────────

export const getOraclePrice = (assetId: string): Promise<PriceDto> =>
  apiFetch<PriceDto>(API_ENDPOINTS.ORACLE_PRICE(assetId));

// ─── Recovery Strategy ────────────────────────────────────────────────────────

export const quoteRecovery = (
  body: RecoveryQuoteRequest
): Promise<RecoveryPlanDto> =>
  apiFetch<RecoveryPlanDto>(API_ENDPOINTS.RECOVERY_QUOTE, {
    method: "POST",
    body: JSON.stringify(body),
  });

export const optimizeRecovery = (
  body: RecoveryOptimizeRequest
): Promise<RecoveryRecommendationResponse> =>
  apiFetch<RecoveryRecommendationResponse>(API_ENDPOINTS.RECOVERY_OPTIMIZE, {
    method: "POST",
    body: JSON.stringify(body),
  });

export const getRecoveryRecommendation = (
  recoveryId: string
): Promise<RecoveryRecommendationResponse> =>
  apiFetch<RecoveryRecommendationResponse>(
    API_ENDPOINTS.RECOVERY_BY_ID(recoveryId)
  );

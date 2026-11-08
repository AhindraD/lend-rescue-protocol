// ─── Shared & Provenance ──────────────────────────────────────────────────────

export type Provenance =
  | "Live"
  | "SurfpoolSimulated"
  | "MockOracle"
  | "Cached"
  | "Demo";

export interface ProvenanceInfo {
  source: Provenance;
  timestamp_ms: number;
  staleness_ms?: number;
}

// ─── Health & Readiness ───────────────────────────────────────────────────────

export interface HealthResponse {
  status: "ok" | "degraded" | "error";
  version: string;
  uptime_secs: number;
}

export interface ReadyResponse {
  ready: boolean;
  checks: Record<string, boolean>;
}

// ─── Markets ──────────────────────────────────────────────────────────────────

export interface MarketReserve {
  asset_id: string;
  symbol: string;
  mint: string;
  deposit_limit_usd?: string;
  borrow_limit_usd?: string;
  liquidation_threshold_bps: number;
  loan_to_value_bps: number;
}

export interface Market {
  id: string;
  name: string;
  protocol: string;
  tvl_usd: string;
  reserve_count: number;
  reserves?: MarketReserve[];
  provenance: ProvenanceInfo;
}

export interface ListMarketsResponse {
  markets: Market[];
  provenance: ProvenanceInfo;
}

// ─── Positions ────────────────────────────────────────────────────────────────

export type RiskLevel = "Safe" | "Warning" | "Critical" | "Unknown";

export interface TokenAmount {
  mint: string;
  symbol: string;
  amount_ui: string;
  value_usd: string;
}

export interface Position {
  id: string;
  owner: string;
  market_id: string;
  collateral: TokenAmount[];
  debt: TokenAmount[];
  health_factor: string | null;
  liquidation_threshold: string | null;
  risk_level: RiskLevel;
  provenance: ProvenanceInfo;
  multi_asset: boolean;
  unsupported_reason?: string;
}

export interface ListPositionsResponse {
  positions: Position[];
  total: number;
  provenance: ProvenanceInfo;
}

// ─── Risk Assessment ──────────────────────────────────────────────────────────

export interface RiskAssessment {
  position_id: string;
  health_factor: string | null;
  liquidation_price: string | null;
  risk_level: RiskLevel;
  risk_score: number;
  time_to_liquidation_estimate?: string;
  provenance: ProvenanceInfo;
  unsupported: boolean;
  unsupported_reason?: string;
}

// ─── Oracle Prices ────────────────────────────────────────────────────────────

export interface OraclePrice {
  asset_id: string;
  symbol: string;
  price_usd: string;
  confidence: string;
  exponent: number;
  publish_time_ms: number;
  feed_account?: string;
  provenance: ProvenanceInfo;
}

// ─── Recovery & Strategy ──────────────────────────────────────────────────────

export type RecoveryActionKind =
  | "RepayDebt"
  | "AddCollateral"
  | "SwapCollateral"
  | "PartialClose";

export interface RecoveryAction {
  kind: RecoveryActionKind;
  description: string;
  token_in?: TokenAmount;
  token_out?: TokenAmount;
  indicative_jupiter_quote?: JupiterQuoteSummary;
}

export interface JupiterQuoteSummary {
  input_mint: string;
  output_mint: string;
  in_amount: string;
  out_amount: string;
  price_impact_pct: string;
  route_plan: string[];
  note: "Indicative Jupiter quote";
}

export interface RecoveryCandidate {
  rank: number;
  strategy_id: string;
  label: string;
  description: string;
  estimated_health_factor_after: string | null;
  actions: RecoveryAction[];
  feasibility_score: number;
  provenance: ProvenanceInfo;
}

export interface RecoveryQuoteRequest {
  position_id: string;
  target_health_factor?: string;
}

export interface RecoveryQuoteResponse {
  recovery_id: string;
  position_id: string;
  candidates: RecoveryCandidate[];
  computed_at_ms: number;
  provenance: ProvenanceInfo;
  unsupported: boolean;
  unsupported_reason?: string;
}

export interface OptimizeRecoveryRequest {
  position_id: string;
  target_health_factor?: string;
  constraints?: {
    max_slippage_bps?: number;
    prefer_no_swap?: boolean;
  };
}

export interface RecoveryRecommendation {
  recovery_id: string;
  position_id: string;
  recommended: RecoveryCandidate | null;
  all_candidates: RecoveryCandidate[];
  computed_at_ms: number;
  provenance: ProvenanceInfo;
  unsupported: boolean;
  unsupported_reason?: string;
}

// ─── Error Envelope ───────────────────────────────────────────────────────────

export interface ApiError {
  error: string;
  code: string;
  request_id?: string;
  details?: Record<string, unknown>;
}


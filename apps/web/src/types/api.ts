// ─── Token & Pricing Types ───────────────────────────────────────────────────

export interface TokenAmountDto {
  raw: string;
  decimals: number;
  symbol?: string;
  formatted: string;
}

export interface PriceDto {
  mantissa: string;
  exponent: number;
  confidence: string;
  formatted: string;
  timestamp: number;
  slot: number;
}

// ─── Health & Readiness ───────────────────────────────────────────────────────

export interface HealthResponse {
  status: string;
  service: string;
  version: string;
}

export interface ReadyResponse {
  ready: boolean;
  database: string;
  demoMode: boolean;
  timestamp: number;
}

// ─── Markets ──────────────────────────────────────────────────────────────────

export interface MarketResponse {
  marketId: number;
  authority: string;
  collateralAsset: string;
  collateralDecimals: number;
  debtAsset: string;
  debtDecimals: number;
  liquidationThresholdBps: number;
  demoSwapFeeBps: number;
  isPaused: boolean;
  dataSource: string;
  observedAt: number;
}

// ─── Position Types ───────────────────────────────────────────────────────────

export interface PositionDepositDto {
  reserveAddress: string;
  mintAddress: string;
  assetSymbol: string;
  decimals: number;
  amount: TokenAmountDto;
  marketValueUsd: string;
}

export interface PositionBorrowDto {
  reserveAddress: string;
  mintAddress: string;
  assetSymbol: string;
  decimals: number;
  amount: TokenAmountDto;
  marketValueUsd: string;
}

export interface ProtocolRiskMetricsDto {
  borrowLimitUsd: string;
  liquidationThresholdUsd: string;
  totalDepositUsd: string;
  totalBorrowUsd: string;
  loanToValueBps: number;
  liquidationThresholdBps: number;
  borrowUtilizationBps: number;
  netAccountValueUsd: string;
  isProtocolReported: boolean;
}

export interface PositionResponse {
  id: string;
  owner: string;
  collateralAsset: string;
  collateralAmount: TokenAmountDto;
  debtAsset: string;
  debtAmount: TokenAmountDto;
  liquidationThresholdBps: number;
  protocolSource: string;
  provenanceLabel: string;
  marketId?: number;
  collateralPositions?: PositionDepositDto[];
  debtPositions?: PositionBorrowDto[];
  protocolMetrics?: ProtocolRiskMetricsDto;
  marketAddress?: string;
  observationSlot?: number;
}

export interface PositionFilterQuery {
  owner?: string;
  protocol?: string;
  network?: string;
  riskBand?: string;
}

// ─── Oracle Types ─────────────────────────────────────────────────────────────

export interface OraclePriceResponse {
  assetId: string;
  price: PriceDto;
  isStale: boolean;
  isValid: boolean;
  dataSource: string;
  ageSecs: number;
}

// ─── Risk Assessment ──────────────────────────────────────────────────────────

export type RiskBand = "Safe" | "Moderate" | "Warning" | "Critical" | "Unknown";
export type RiskLevel = RiskBand | string;

export interface RiskAssessmentResponse {
  positionId: string;
  healthFactor: string | null;
  healthFactorScaled: number | null;
  riskBand: string;
  riskBandLabel: string;
  collateralValueFormatted: string;
  collateralValueRaw: string;
  debtValueFormatted: string;
  debtValueRaw: string;
  borrowCapacityFormatted: string;
  borrowCapacityRaw: string;
  liquidationPrice: PriceDto | null;
  distanceToLiquidationBps: number | null;
  dataProvenance: string;
  evaluatedAt: number;
  protocolMetrics?: ProtocolRiskMetricsDto;
}

// ─── Recovery & Plan Types ────────────────────────────────────────────────────

export interface ScoreBreakdownDto {
  capitalEfficiency: number;
  safetyBuffer: number;
  feePenalty: number;
  slippagePenalty: number;
  totalScore: number;
}

export interface SwapRouteStepDto {
  ammKey?: string;
  label?: string;
  inputMint: string;
  outputMint: string;
  inAmount: string;
  outAmount: string;
  feeAmount: string;
  feeMint?: string;
  percent: number;
}

export interface SwapQuoteDto {
  provider: string;
  inputMint: string;
  outputMint: string;
  inAmount: TokenAmountDto;
  expectedOutAmount: TokenAmountDto;
  planningMinOutAmount: TokenAmountDto;
  feeBps: number;
  feeMint?: string;
  platformFeeBps?: number;
  platformFeeAmount?: string;
  router: string;
  mode: string;
  priceImpactPct?: string;
  routePlan: SwapRouteStepDto[];
  observedAt: number;
  validUntil: number;
  isIndicative: boolean;
  isValid: boolean;
}

export interface RecoveryPlanDto {
  action: string;
  actionLabel: string;
  inputAmount: TokenAmountDto;
  expectedOutputAmount?: TokenAmountDto;
  minOutputAmount?: TokenAmountDto;
  debtRepaid: TokenAmountDto;
  collateralSpent: TokenAmountDto;
  collateralAdded: TokenAmountDto;
  expectedFeeFormatted: string;
  expectedFeeRaw: string;
  estimatedSlippageBps: number;
  postHealthFactor: string;
  postHealthFactorScaled: number;
  safetyBufferBps: number;
  isFeasible: boolean;
  rejectionReason?: string;
  totalCostFormatted: string;
  totalCostRaw: string;
  score: number;
  scoreBreakdown: ScoreBreakdownDto;
  swapQuote?: SwapQuoteDto;
}

export interface RecoveryRecommendationResponse {
  recoveryId: string;
  positionId: string;
  status: string;
  preHealthFactor: string;
  targetHealthFactor: string;
  recommendedPlan: RecoveryPlanDto | null;
  alternativePlans: RecoveryPlanDto[];
  disclaimer: string;
  createdAt: number;
  expiresAt: number;
}

export interface RecoveryQuoteRequest {
  positionId?: string;
  action?: string;
  targetHealthFactor?: string;
  maxDebtRepayment?: string;
  maxCollateralSpend?: string;
  maxSlippageBps?: number;
  inputMint?: string;
  outputMint?: string;
  amount?: string;
  refresh?: boolean;
}

export interface RecoveryOptimizeRequest {
  positionId: string;
  targetHealthFactor?: string;
  allowedActions?: string[];
  maxDebtRepayment?: string;
  maxCollateralSpend?: string;
  maxSlippageBps?: number;
  objective?: string;
}

// ─── Error Envelope ───────────────────────────────────────────────────────────

export interface ErrorPayload {
  code: string;
  message: string;
  requestId?: string;
}

export interface ErrorResponse {
  error: ErrorPayload;
}

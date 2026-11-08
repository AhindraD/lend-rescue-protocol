/**
 * Data Transfer Objects and internal API contracts for Kamino Lending Adapter.
 */

export interface KaminoReserveResponse {
  reserveId: string;
  liquidityMint: string;
  collateralMint: string;
  tokenSymbol: string;
  decimals: number;
  loanToValueBps: number;
  liquidationThresholdBps: number;
  liquidationBonusBps: number;
  availableLiquidity: string;
  totalDeposits: string;
  totalBorrows: string;
  borrowInterestRateBps: number;
  supplyInterestRateBps: number;
  oracleAddress: string;
  lastUpdateSlot: number;
  status: string;
}

export interface KaminoMarketResponse {
  marketId: string;
  owner: string;
  authority: string;
  programId: string;
  network: string;
  observationSlot: number;
  observedAt: string;
  reserves: KaminoReserveResponse[];
}

export interface KaminoDepositEntry {
  reserveAddress: string;
  mintAddress: string;
  tokenSymbol: string;
  decimals: number;
  rawAmount: string;
  marketValueUsd: string;
}

export interface KaminoBorrowEntry {
  reserveAddress: string;
  mintAddress: string;
  tokenSymbol: string;
  decimals: number;
  rawAmount: string;
  marketValueUsd: string;
}

export interface KaminoProtocolMetrics {
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

export interface KaminoObligationResponse {
  obligationId: string;
  owner: string;
  marketId: string;
  observationSlot: number;
  observedAt: string;
  protocolSource: "KaminoMainnet" | "SurfpoolSimulated";
  deposits: KaminoDepositEntry[];
  borrows: KaminoBorrowEntry[];
  protocolMetrics: KaminoProtocolMetrics;
}

export interface KaminoErrorResponse {
  error: string;
  code: string;
  details?: unknown;
}

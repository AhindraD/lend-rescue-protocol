/**
 * Configuration for Kamino Lending Adapter.
 */

export interface AdapterConfig {
  bindHost: string;
  bindPort: number;
  solanaRpcUrl: string;
  solanaNetwork: string;
  programId: string;
  defaultMarket: string;
  requestTimeoutMs: number;
  maxSlotDrift: number;
  cacheTtlMs: number;
}

export function loadConfig(): AdapterConfig {
  return {
    bindHost: process.env.KAMINO_ADAPTER_BIND_HOST || "127.0.0.1",
    bindPort: parseInt(process.env.KAMINO_ADAPTER_BIND_PORT || "3002", 10),
    solanaRpcUrl:
      process.env.SOLANA_RPC_URL ||
      process.env.SURFPOOL_RPC_URL ||
      "http://127.0.0.1:8899",
    solanaNetwork: process.env.SOLANA_NETWORK || "surfpool",
    programId:
      process.env.KAMINO_PROGRAM_ID || "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
    defaultMarket:
      process.env.KAMINO_DEFAULT_MARKET ||
      process.env.KAMINO_MARKET_ADDRESS ||
      "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
    requestTimeoutMs: parseInt(
      process.env.KAMINO_REQUEST_TIMEOUT_MS || "10000",
      10
    ),
    maxSlotDrift: parseInt(process.env.KAMINO_MAX_SLOT_DRIFT || "50", 10),
    cacheTtlMs: parseInt(process.env.KAMINO_CACHE_TTL_MS || "5000", 10),
  };
}


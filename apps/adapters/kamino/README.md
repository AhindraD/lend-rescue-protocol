# Kamino Lending Read-Only Adapter

Isolated TypeScript micro-service providing production-oriented, read-only integration with Kamino Lending on Solana.

Uses official `@kamino-finance/klend-sdk@13.0.6` while isolating its dependencies (`@solana/kit@2.3.0`, `@coral-xyz/anchor@0.28.0`) from the root Anchor 1.2.2 workspace.

## Read-Only Security Boundary

This adapter is strictly advisory and read-only:
- **Allowed**: Read lending markets, read reserves, read user obligations, normalize multi-asset positions, preserve official protocol risk metrics.
- **Forbidden**: Deposits, withdrawals, borrows, repayments, liquidations, transaction signing, private key handling, and external broadcasts.

## Configuration

The adapter is configured via environment variables:

| Variable | Default | Description |
|---|---|---|
| `KAMINO_ADAPTER_BIND_HOST` | `127.0.0.1` | Local bind interface (never bind public) |
| `KAMINO_ADAPTER_BIND_PORT` | `3002` | Local HTTP port |
| `SOLANA_RPC_URL` | `http://127.0.0.1:8899` | Solana JSON-RPC endpoint |
| `SOLANA_NETWORK` | `surfpool` | Solana network identifier (`mainnet`, `surfpool`, `devnet`) |
| `KAMINO_PROGRAM_ID` | `KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD` | Kamino Lend v2 Program ID |
| `KAMINO_DEFAULT_MARKET` | `7u3HeHxatcAACvdFtHGryaZWwZxKhqWisdHJaJBYbugj` | Kamino Main Market |
| `KAMINO_REQUEST_TIMEOUT_MS` | `10000` | RPC timeout in milliseconds |
| `KAMINO_MAX_SLOT_DRIFT` | `50` | Maximum acceptable slot drift across queries |
| `KAMINO_CACHE_TTL_MS` | `5000` | In-memory market reserve cache TTL |

## Supported Markets

- **Kamino Main Market (`mainnet-beta` / `surfpool`)**:
  - Address: `7u3HeHxatcAACvdFtHGryaZWwZxKhqWisdHJaJBYbugj`
  - Reserves: SOL, USDC, USDT, bSOL, mSOL, JitoSOL, etc.
- **Custom / Staging Markets**:
  - Any valid Kamino Lend v2 market address can be queried dynamically.

## Internal HTTP API Contract

All endpoints bind to `127.0.0.1` and return JSON.

### 1. Health & Status
```http
GET /health
```
Response:
```json
{
  "status": "ok",
  "version": "0.1.0",
  "sdkVersion": "13.0.6",
  "network": "surfpool",
  "readOnly": true,
  "timestamp": "2026-11-08T11:45:00.000Z"
}
```

### 2. Market Snapshot
```http
GET /markets/:market_id
```
Returns normalized market configuration, authority, program ID, and array of reserves.

### 3. Reserves
```http
GET /reserves/:market_id
```
Returns array of normalized reserve parameters (LTV, liquidation threshold, liquidity, APRs, oracle address).

### 4. Obligations
```http
GET /obligations/:market_id/:owner
GET /obligations/:market_id/:owner/:obligation_id
```
Returns normalized obligation snapshots containing:
- Multi-asset collateral deposits (`rawAmount`, `decimals`, `marketValueUsd`)
- Multi-asset debt borrows (`rawAmount`, `decimals`, `marketValueUsd`)
- Official protocol risk metrics (`borrowLimitUsd`, `liquidationThresholdUsd`, `loanToValueBps`, `borrowUtilizationBps`, `netAccountValueUsd`, `isProtocolReported: true`)
- Slot metadata and data provenance (`KaminoMainnet` or `SurfpoolSimulated`).

## Testing

```bash
# Run unit and integration tests
pnpm test

# Run TypeScript type check
pnpm run typecheck

# Start local adapter daemon
pnpm start
```


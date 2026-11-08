import { createSolanaRpc, address, Address } from "@solana/kit";
import {
  KaminoMarket,
  KaminoReserve,
  KaminoObligation,
  DEFAULT_RECENT_SLOT_DURATION_MS,
  getCurrentLedgerInstant,
  LedgerInstant,
} from "@kamino-finance/klend-sdk";
import { AdapterConfig } from "./config";
import { NotFoundError, RpcError, TimeoutError, ValidationError } from "./errors";

interface CachedMarket {
  market: KaminoMarket;
  fetchedAt: number;
  slot: number;
}

/**
 * Pure read-only client for Kamino Lending.
 * Enforces strict non-custodial and read-only boundaries.
 */
export class KaminoClient {
  private readonly config: AdapterConfig;
  private readonly rpc: ReturnType<typeof createSolanaRpc>;
  private readonly programId: Address;
  private readonly marketCache: Map<string, CachedMarket> = new Map();

  constructor(config: AdapterConfig) {
    this.config = config;
    this.rpc = createSolanaRpc(config.solanaRpcUrl);
    try {
      this.programId = address(config.programId);
    } catch (e: any) {
      throw new ValidationError(`Invalid program ID '${config.programId}': ${e.message}`);
    }
  }

  /**
   * Helper to execute a promise with timeout.
   */
  private async withTimeout<T>(promise: Promise<T>, operation: string): Promise<T> {
    let timeoutId: NodeJS.Timeout;
    const timeoutPromise = new Promise<never>((_, reject) => {
      timeoutId = setTimeout(() => {
        reject(
          new TimeoutError(
            `Operation '${operation}' timed out after ${this.config.requestTimeoutMs}ms`
          )
        );
      }, this.config.requestTimeoutMs);
    });

    try {
      return await Promise.race([promise, timeoutPromise]);
    } catch (err: any) {
      if (err instanceof TimeoutError) throw err;
      throw new RpcError(`RPC operation '${operation}' failed: ${err.message}`, err);
    } finally {
      clearTimeout(timeoutId!);
    }
  }

  /**
   * Validates and parses a Solana public key address string.
   */
  public parseAddress(addrStr: string, fieldName = "address"): Address {
    try {
      return address(addrStr);
    } catch (e: any) {
      throw new ValidationError(
        `Invalid ${fieldName} '${addrStr}': must be a valid base58 Solana address`
      );
    }
  }

  /**
   * Fetches the current slot from the RPC node.
   */
  public async getSlot(): Promise<number> {
    return this.withTimeout(
      (async () => {
        const slotBig = await this.rpc.getSlot().send();
        return Number(slotBig);
      })(),
      "getSlot"
    );
  }

  /**
   * Fetches or resolves current ledger instant.
   */
  public async getLedgerInstant(): Promise<LedgerInstant> {
    return this.withTimeout(
      (async () => {
        try {
          return await getCurrentLedgerInstant(this.rpc as any);
        } catch {
          const slotBig = await this.rpc.getSlot().send();
          return {
            slot: slotBig as any,
            blockTime: BigInt(Math.floor(Date.now() / 1000)) as any,
          };
        }
      })(),
      "getLedgerInstant"
    );
  }

  /**
   * Loads a KaminoMarket and its reserves by market address.
   */
  public async loadMarket(marketAddressStr: string): Promise<{ market: KaminoMarket; slot: number }> {
    const marketAddr = this.parseAddress(marketAddressStr, "marketId");
    const now = Date.now();
    const cached = this.marketCache.get(marketAddr);

    if (cached && now - cached.fetchedAt < this.config.cacheTtlMs) {
      return { market: cached.market, slot: cached.slot };
    }

    const { market, slot } = await this.withTimeout(
      (async () => {
        const currentSlot = Number(await this.rpc.getSlot().send());
        const loaded = await KaminoMarket.load(
          this.rpc as any,
          marketAddr,
          DEFAULT_RECENT_SLOT_DURATION_MS,
          this.programId,
          true
        );
        if (!loaded) {
          throw new NotFoundError(`Kamino market '${marketAddressStr}' not found`);
        }
        return { market: loaded, slot: currentSlot };
      })(),
      `loadMarket(${marketAddressStr})`
    );

    this.marketCache.set(marketAddr, { market, fetchedAt: now, slot });
    return { market, slot };
  }

  /**
   * Loads reserves for a given market.
   */
  public async loadReserves(marketAddressStr: string): Promise<{ reserves: KaminoReserve[]; slot: number }> {
    const { market, slot } = await this.loadMarket(marketAddressStr);
    return { reserves: market.getReserves(), slot };
  }

  /**
   * Fetches all obligations for a specific owner in a given lending market.
   */
  public async getUserObligations(
    marketAddressStr: string,
    ownerAddressStr: string
  ): Promise<{ obligations: KaminoObligation[]; slot: number }> {
    const ownerAddr = this.parseAddress(ownerAddressStr, "owner");
    const { market, slot } = await this.loadMarket(marketAddressStr);
    const instant = await this.getLedgerInstant();

    const obligations = await this.withTimeout(
      market.getAllUserObligations(ownerAddr, instant),
      `getUserObligations(${ownerAddressStr})`
    );

    return { obligations, slot };
  }

  /**
   * Fetches a specific obligation by its address and verifies owner match.
   */
  public async getObligation(
    marketAddressStr: string,
    ownerAddressStr: string,
    obligationAddressStr: string
  ): Promise<{ obligation: KaminoObligation; slot: number }> {
    const ownerAddr = this.parseAddress(ownerAddressStr, "owner");
    const obligationAddr = this.parseAddress(obligationAddressStr, "obligationId");
    const { market, slot } = await this.loadMarket(marketAddressStr);

    const obligation = await this.withTimeout(
      KaminoObligation.load(market, obligationAddr),
      `getObligation(${obligationAddressStr})`
    );

    if (!obligation) {
      throw new NotFoundError(
        `Obligation '${obligationAddressStr}' not found in market '${marketAddressStr}'`
      );
    }

    if (obligation.state.owner.toString() !== ownerAddr.toString()) {
      throw new ValidationError(
        `Obligation '${obligationAddressStr}' owner mismatch: expected '${ownerAddressStr}', found '${obligation.state.owner.toString()}'`
      );
    }

    return { obligation, slot };
  }
}

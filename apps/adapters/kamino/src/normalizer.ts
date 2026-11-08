import {
  KaminoMarket,
  KaminoReserve,
  KaminoObligation,
  LedgerInstant,
} from "@kamino-finance/klend-sdk";
import {
  KaminoMarketResponse,
  KaminoReserveResponse,
  KaminoObligationResponse,
  KaminoDepositEntry,
  KaminoBorrowEntry,
  KaminoProtocolMetrics,
} from "./contracts";
import { InconsistentSlotError } from "./errors";

/**
 * Normalizes Kamino lending protocol structures into LendRescue contracts.
 */
export class KaminoNormalizer {
  /**
   * Determine data provenance label based on network and environment.
   */
  public static resolveProtocolSource(
    network: string
  ): "KaminoMainnet" | "SurfpoolSimulated" {
    const net = network.toLowerCase();
    if (net.includes("mainnet")) {
      return "KaminoMainnet";
    }
    return "SurfpoolSimulated";
  }

  /**
   * Validates slot consistency across multiple observations.
   */
  public static checkSlotDrift(
    baseSlot: number,
    targetSlot: number,
    maxDrift: number
  ): void {
    const drift = Math.abs(baseSlot - targetSlot);
    if (drift > maxDrift) {
      throw new InconsistentSlotError(
        `Slot drift ${drift} exceeds maximum allowable drift ${maxDrift} (base: ${baseSlot}, target: ${targetSlot})`,
        { baseSlot, targetSlot, drift, maxDrift }
      );
    }
  }

  /**
   * Normalizes a KaminoReserve instance into a KaminoReserveResponse DTO.
   */
  public static normalizeReserve(
    reserve: KaminoReserve,
    instant?: LedgerInstant
  ): KaminoReserveResponse {
    const config = reserve.state.config;
    const decimals = reserve.getMintDecimals();

    // Scale percentages to basis points (e.g. 75% -> 7500 bps)
    const ltvBps = (config.loanToValuePct || 0) * 100;
    const liqThresholdBps = (config.liquidationThresholdPct || 0) * 100;
    const liqBonusBps = config.minLiquidationBonusBps || 0;

    // Available liquidity, total deposits, and total borrows in raw units
    const availableRaw = reserve.getLiquidityAvailableAmount();
    const depositsRaw = reserve.getTotalSupply();
    const borrowsRaw = reserve.getBorrowedAmount();

    // Rates in basis points
    let borrowAprBps = 0;
    let supplyAprBps = 0;
    const evalInstant: LedgerInstant = instant ?? ({
      slot: BigInt(reserve.state.lastUpdate.slot.toString()) as any,
      blockTime: BigInt(Math.floor(Date.now() / 1000)) as any,
    } as LedgerInstant);

    try {
      const bApr = reserve.calculateBorrowAPR(evalInstant, 0);
      borrowAprBps = Math.round(bApr * 10000);
    } catch {
      borrowAprBps = 0;
    }
    try {
      const sApr = reserve.calculateSupplyAPR(evalInstant, 0);
      supplyAprBps = Math.round(sApr * 10000);
    } catch {
      supplyAprBps = 0;
    }

    // Oracle address (Pyth, Scope, or Switchboard)
    let oracleAddress = "";
    if (config.tokenInfo?.pythConfiguration?.price) {
      oracleAddress = config.tokenInfo.pythConfiguration.price.toString();
    } else if (config.tokenInfo?.scopeConfiguration?.priceFeed) {
      oracleAddress = config.tokenInfo.scopeConfiguration.priceFeed.toString();
    } else if (config.tokenInfo?.switchboardConfiguration?.priceAggregator) {
      oracleAddress =
        config.tokenInfo.switchboardConfiguration.priceAggregator.toString();
    }

    return {
      reserveId: reserve.address.toString(),
      liquidityMint: reserve.getLiquidityMint().toString(),
      collateralMint: reserve.getCTokenMint().toString(),
      tokenSymbol: reserve.getTokenSymbol() || "UNKNOWN",
      decimals,
      loanToValueBps: ltvBps,
      liquidationThresholdBps: liqThresholdBps,
      liquidationBonusBps: liqBonusBps,
      availableLiquidity: availableRaw.toFixed(0),
      totalDeposits: depositsRaw.toFixed(0),
      totalBorrows: borrowsRaw.toFixed(0),
      borrowInterestRateBps: borrowAprBps,
      supplyInterestRateBps: supplyAprBps,
      oracleAddress,
      lastUpdateSlot: Number(reserve.state.lastUpdate.slot),
      status: config.status === 0 ? "active" : "disabled",
    };
  }

  /**
   * Normalizes a KaminoMarket instance into a KaminoMarketResponse DTO.
   */
  public static normalizeMarket(
    market: KaminoMarket,
    network: string,
    observationSlot: number,
    instant?: LedgerInstant
  ): KaminoMarketResponse {
    const reserves: KaminoReserveResponse[] = [];
    for (const res of market.getReserves()) {
      reserves.push(this.normalizeReserve(res, instant));
    }

    return {
      marketId: market.address.toString(),
      owner: market.state.lendingMarketOwner.toString(),
      authority: market.getLendingMarketAuthority().toString(),
      programId: market.programId.toString(),
      network,
      observationSlot,
      observedAt: new Date().toISOString(),
      reserves,
    };
  }

  /**
   * Normalizes a KaminoObligation instance into a KaminoObligationResponse DTO.
   */
  public static normalizeObligation(
    obligation: KaminoObligation,
    network: string,
    observationSlot: number
  ): KaminoObligationResponse {
    const deposits: KaminoDepositEntry[] = [];
    const borrows: KaminoBorrowEntry[] = [];

    // Extract multi-asset collateral deposits
    for (const dep of obligation.getDeposits()) {
      const reserve = obligation.market.getReserveByAddress(dep.reserveAddress);
      const symbol = reserve ? reserve.getTokenSymbol() : "UNKNOWN";
      const decimals = reserve ? reserve.getMintDecimals() : 9;

      // dep.amount is in base units
      deposits.push({
        reserveAddress: dep.reserveAddress.toString(),
        mintAddress: dep.mintAddress.toString(),
        tokenSymbol: symbol,
        decimals,
        rawAmount: dep.amount.toFixed(0),
        marketValueUsd: dep.marketValueRefreshed.toFixed(6),
      });
    }

    // Extract multi-asset debt borrows
    for (const brw of obligation.getBorrows()) {
      const reserve = obligation.market.getReserveByAddress(brw.reserveAddress);
      const symbol = reserve ? reserve.getTokenSymbol() : "UNKNOWN";
      const decimals = reserve ? reserve.getMintDecimals() : 6;

      // brw.amount is in base units
      borrows.push({
        reserveAddress: brw.reserveAddress.toString(),
        mintAddress: brw.mintAddress.toString(),
        tokenSymbol: symbol,
        decimals,
        rawAmount: brw.amount.toFixed(0),
        marketValueUsd: brw.marketValueRefreshed.toFixed(6),
      });
    }

    // Calculate official protocol metrics
    const stats = obligation.refreshedStats;
    const ltvDecimal = obligation.loanToValue();
    const liqLtvDecimal = obligation.liquidationLtv();

    const ltvBps = Math.round(ltvDecimal.mul(10000).toNumber());
    const liqThresholdBps = Math.round(liqLtvDecimal.mul(10000).toNumber());

    let borrowUtilBps = 0;
    if (stats.borrowLimit && !stats.borrowLimit.isZero()) {
      const util = stats.userTotalBorrow.div(stats.borrowLimit);
      borrowUtilBps = Math.round(util.mul(10000).toNumber());
    }

    const netAccountValue = stats.userTotalDeposit.minus(stats.userTotalBorrow);

    const protocolMetrics: KaminoProtocolMetrics = {
      borrowLimitUsd: stats.borrowLimit.toFixed(6),
      liquidationThresholdUsd: stats.borrowLiquidationLimit.toFixed(6),
      totalDepositUsd: stats.userTotalDeposit.toFixed(6),
      totalBorrowUsd: stats.userTotalBorrow.toFixed(6),
      loanToValueBps: ltvBps,
      liquidationThresholdBps: liqThresholdBps,
      borrowUtilizationBps: borrowUtilBps,
      netAccountValueUsd: netAccountValue.toFixed(6),
      isProtocolReported: true,
    };

    return {
      obligationId: obligation.obligationAddress.toString(),
      owner: obligation.state.owner.toString(),
      marketId: obligation.market.address.toString(),
      observationSlot,
      observedAt: new Date().toISOString(),
      protocolSource: this.resolveProtocolSource(network),
      deposits,
      borrows,
      protocolMetrics,
    };
  }
}

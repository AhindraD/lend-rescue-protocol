import { expect } from "chai";
import Decimal from "decimal.js";
import { KaminoNormalizer } from "../src/normalizer";
import { InconsistentSlotError } from "../src/errors";

describe("KaminoNormalizer", () => {
  describe("resolveProtocolSource", () => {
    it("maps mainnet environments to KaminoMainnet", () => {
      expect(KaminoNormalizer.resolveProtocolSource("mainnet")).to.equal(
        "KaminoMainnet"
      );
      expect(KaminoNormalizer.resolveProtocolSource("mainnet-beta")).to.equal(
        "KaminoMainnet"
      );
      expect(
        KaminoNormalizer.resolveProtocolSource("https://api.mainnet-beta.solana.com")
      ).to.equal("KaminoMainnet");
    });

    it("maps non-mainnet environments to SurfpoolSimulated", () => {
      expect(KaminoNormalizer.resolveProtocolSource("surfpool")).to.equal(
        "SurfpoolSimulated"
      );
      expect(KaminoNormalizer.resolveProtocolSource("devnet")).to.equal(
        "SurfpoolSimulated"
      );
      expect(KaminoNormalizer.resolveProtocolSource("localnet")).to.equal(
        "SurfpoolSimulated"
      );
    });
  });

  describe("checkSlotDrift", () => {
    it("permits slot drift within maximum tolerance", () => {
      expect(() => {
        KaminoNormalizer.checkSlotDrift(1000, 1025, 50);
      }).not.to.throw();

      expect(() => {
        KaminoNormalizer.checkSlotDrift(1050, 1000, 50);
      }).not.to.throw();
    });

    it("throws InconsistentSlotError when slot drift exceeds tolerance", () => {
      expect(() => {
        KaminoNormalizer.checkSlotDrift(1000, 1051, 50);
      }).to.throw(InconsistentSlotError, /Slot drift 51 exceeds maximum allowable drift 50/);
    });
  });

  describe("normalizeReserve", () => {
    it("normalizes reserve risk parameters, decimals, and metrics without loss of precision", () => {
      const mockReserve: any = {
        address: "Reserve1111111111111111111111111111111111111",
        state: {
          config: {
            loanToValuePct: 75,
            liquidationThresholdPct: 85,
            minLiquidationBonusBps: 500,
            status: 0,
            tokenInfo: {
              pythConfiguration: {
                price: "PythFeed111111111111111111111111111111111111",
              },
            },
          },
          lastUpdate: {
            slot: { toString: () => "250000000" },
          },
        },
        getMintDecimals: () => 9,
        getLiquidityMint: () => "So11111111111111111111111111111111111111112",
        getCTTokenMint: () => "cSo11111111111111111111111111111111111111112",
        getCTokenMint: () => "cSo11111111111111111111111111111111111111112",
        getTokenSymbol: () => "SOL",
        getLiquidityAvailableAmount: () => new Decimal("5000000000000"),
        getTotalSupply: () => new Decimal("12000000000000"),
        getBorrowedAmount: () => new Decimal("7000000000000"),
        calculateBorrowAPR: () => 0.0825, // 8.25%
        calculateSupplyAPR: () => 0.0415, // 4.15%
      };

      const normalized = KaminoNormalizer.normalizeReserve(mockReserve);

      expect(normalized.reserveId).to.equal(
        "Reserve1111111111111111111111111111111111111"
      );
      expect(normalized.tokenSymbol).to.equal("SOL");
      expect(normalized.decimals).to.equal(9);
      expect(normalized.loanToValueBps).to.equal(7500);
      expect(normalized.liquidationThresholdBps).to.equal(8500);
      expect(normalized.liquidationBonusBps).to.equal(500);
      expect(normalized.availableLiquidity).to.equal("5000000000000");
      expect(normalized.totalDeposits).to.equal("12000000000000");
      expect(normalized.totalBorrows).to.equal("7000000000000");
      expect(normalized.borrowInterestRateBps).to.equal(825);
      expect(normalized.supplyInterestRateBps).to.equal(415);
      expect(normalized.oracleAddress).to.equal(
        "PythFeed111111111111111111111111111111111111"
      );
      expect(normalized.status).to.equal("active");
    });
  });

  describe("normalizeObligation", () => {
    it("normalizes multi-asset deposits, multi-asset borrows, and protocol metrics", () => {
      const mockObligation: any = {
        obligationAddress: "Obligation1111111111111111111111111111111111",
        state: {
          owner: "Owner111111111111111111111111111111111111111",
        },
        market: {
          address: "Market11111111111111111111111111111111111111",
          getReserveByAddress: (addr: string) => {
            if (addr === "ReserveSOL") {
              return {
                getTokenSymbol: () => "SOL",
                getMintDecimals: () => 9,
              };
            }
            if (addr === "ReserveUSDC") {
              return {
                getTokenSymbol: () => "USDC",
                getMintDecimals: () => 6,
              };
            }
            return null;
          },
        },
        getDeposits: () => [
          {
            reserveAddress: "ReserveSOL",
            mintAddress: "MintSOL",
            amount: new Decimal("10000000000"), // 10 SOL
            marketValueRefreshed: new Decimal("1500.00"),
          },
        ],
        getBorrows: () => [
          {
            reserveAddress: "ReserveUSDC",
            mintAddress: "MintUSDC",
            amount: new Decimal("900000000"), // 900 USDC
            marketValueRefreshed: new Decimal("900.00"),
          },
        ],
        refreshedStats: {
          borrowLimit: new Decimal("1125.00"),
          borrowLiquidationLimit: new Decimal("1275.00"),
          userTotalDeposit: new Decimal("1500.00"),
          userTotalBorrow: new Decimal("900.00"),
        },
        loanToValue: () => new Decimal("0.6000"),
        liquidationLtv: () => new Decimal("0.8500"),
      };

      const normalized = KaminoNormalizer.normalizeObligation(
        mockObligation,
        "surfpool",
        250000100
      );

      expect(normalized.obligationId).to.equal(
        "Obligation1111111111111111111111111111111111"
      );
      expect(normalized.owner).to.equal(
        "Owner111111111111111111111111111111111111111"
      );
      expect(normalized.protocolSource).to.equal("SurfpoolSimulated");
      expect(normalized.observationSlot).to.equal(250000100);

      // Deposits verification
      expect(normalized.deposits).to.have.lengthOf(1);
      expect(normalized.deposits[0].tokenSymbol).to.equal("SOL");
      expect(normalized.deposits[0].decimals).to.equal(9);
      expect(normalized.deposits[0].rawAmount).to.equal("10000000000");
      expect(normalized.deposits[0].marketValueUsd).to.equal("1500.000000");

      // Borrows verification
      expect(normalized.borrows).to.have.lengthOf(1);
      expect(normalized.borrows[0].tokenSymbol).to.equal("USDC");
      expect(normalized.borrows[0].decimals).to.equal(6);
      expect(normalized.borrows[0].rawAmount).to.equal("900000000");
      expect(normalized.borrows[0].marketValueUsd).to.equal("900.000000");

      // Protocol metrics verification
      expect(normalized.protocolMetrics.isProtocolReported).to.be.true;
      expect(normalized.protocolMetrics.totalDepositUsd).to.equal("1500.000000");
      expect(normalized.protocolMetrics.totalBorrowUsd).to.equal("900.000000");
      expect(normalized.protocolMetrics.borrowLimitUsd).to.equal("1125.000000");
      expect(normalized.protocolMetrics.liquidationThresholdUsd).to.equal(
        "1275.000000"
      );
      expect(normalized.protocolMetrics.loanToValueBps).to.equal(6000);
      expect(normalized.protocolMetrics.liquidationThresholdBps).to.equal(8500);
      expect(normalized.protocolMetrics.borrowUtilizationBps).to.equal(8000); // 900 / 1125 = 80.00%
      expect(normalized.protocolMetrics.netAccountValueUsd).to.equal("600.000000");
    });
  });
});


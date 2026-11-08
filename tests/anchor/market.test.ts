import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "@anchor-lang/core";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { getProgram, getProvider } from "./setup";
import {
  getMarketPda,
  getCollateralVaultPda,
  getOraclePda,
} from "./helpers/accounts";
import {
  createSyntheticMint,
  createAndFundUser,
} from "./helpers/fixtures";
import { assertCustomError } from "./helpers/assertions";

describe("LendRescue - Market & Oracle Lifecycle", () => {
  const provider = getProvider();
  const program = getProgram(provider);
  const payer = (provider.wallet as any).payer as Keypair;

  let collateralMint: PublicKey;
  let debtMint: PublicKey;
  let marketId: BN;
  let marketPda: PublicKey;
  let collateralVaultPda: PublicKey;

  before(async () => {
    collateralMint = await createSyntheticMint(provider.connection, payer, 9);
    debtMint = await createSyntheticMint(provider.connection, payer, 6);
    marketId = new BN(1001);
    [marketPda] = getMarketPda(marketId);
    [collateralVaultPda] = getCollateralVaultPda(marketPda);
  });

  it("1. Initializes market successfully with valid parameters", async () => {
    const liquidationThresholdBps = 8_000; // 80%
    const demoSwapFeeBps = 30; // 0.3%

    await program.methods
      .initializeMarket(marketId, liquidationThresholdBps, demoSwapFeeBps)
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
        collateralMint,
        debtMint,
        collateralVault: collateralVaultPda,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchorRentSysvar(),
      })
      .rpc();

    const market = await program.account.marketConfig.fetch(marketPda);
    expect(market.authority.toBase58()).to.equal(payer.publicKey.toBase58());
    expect(market.marketId.toString()).to.equal(marketId.toString());
    expect(market.collateralMint.toBase58()).to.equal(collateralMint.toBase58());
    expect(market.debtMint.toBase58()).to.equal(debtMint.toBase58());
    expect(market.collateralVault.toBase58()).to.equal(collateralVaultPda.toBase58());
    expect(market.collateralDecimals).to.equal(9);
    expect(market.debtDecimals).to.equal(6);
    expect(market.liquidationThresholdBps).to.equal(liquidationThresholdBps);
    expect(market.demoSwapFeeBps).to.equal(demoSwapFeeBps);
    expect(market.isPaused).to.be.false;
  });

  it("2. Fails to initialize market if liquidation threshold is out of bounds", async () => {
    const invalidMarketIdLow = new BN(1002);
    const [pdaLow] = getMarketPda(invalidMarketIdLow);
    const [vaultLow] = getCollateralVaultPda(pdaLow);

    // Below 1,000 bps
    await assertCustomError(
      program.methods
        .initializeMarket(invalidMarketIdLow, 500, 30)
        .accountsStrict({
          authority: payer.publicKey,
          market: pdaLow,
          collateralMint,
          debtMint,
          collateralVault: vaultLow,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchorRentSysvar(),
        })
        .rpc(),
      "InvalidLiquidationThreshold"
    );

    // Above 9,900 bps
    const invalidMarketIdHigh = new BN(1003);
    const [pdaHigh] = getMarketPda(invalidMarketIdHigh);
    const [vaultHigh] = getCollateralVaultPda(pdaHigh);

    await assertCustomError(
      program.methods
        .initializeMarket(invalidMarketIdHigh, 9950, 30)
        .accountsStrict({
          authority: payer.publicKey,
          market: pdaHigh,
          collateralMint,
          debtMint,
          collateralVault: vaultHigh,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchorRentSysvar(),
        })
        .rpc(),
      "InvalidLiquidationThreshold"
    );
  });

  it("3. Fails to initialize market with fee exceeding 1,000 bps", async () => {
    const invalidMarketId = new BN(1004);
    const [pda] = getMarketPda(invalidMarketId);
    const [vault] = getCollateralVaultPda(pda);

    await assertCustomError(
      program.methods
        .initializeMarket(invalidMarketId, 8000, 1500)
        .accountsStrict({
          authority: payer.publicKey,
          market: pda,
          collateralMint,
          debtMint,
          collateralVault: vault,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchorRentSysvar(),
        })
        .rpc(),
      "InvalidFeeBps"
    );
  });

  it("4. Fails to initialize market if collateral mint and debt mint are identical", async () => {
    const invalidMarketId = new BN(1005);
    const [pda] = getMarketPda(invalidMarketId);
    const [vault] = getCollateralVaultPda(pda);

    await assertCustomError(
      program.methods
        .initializeMarket(invalidMarketId, 8000, 30)
        .accountsStrict({
          authority: payer.publicKey,
          market: pda,
          collateralMint,
          debtMint: collateralMint,
          collateralVault: vault,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchorRentSysvar(),
        })
        .rpc(),
      "InvalidMint"
    );
  });

  it("5. Initializes mock collateral and debt oracles successfully", async () => {
    const [collateralOraclePda] = getOraclePda(marketPda, collateralMint);
    const initialPriceCollateral = new BN(150_00000000); // $150.00
    const confidenceCollateral = new BN(500000); // 0.005 ($0.005 = 3.3 bps of $150, < 200 bps)

    await program.methods
      .initializeMockOracle(initialPriceCollateral, -8, confidenceCollateral)
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
        assetMint: collateralMint,
        oracle: collateralOraclePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const collateralOracle = await program.account.mockOracle.fetch(collateralOraclePda);
    expect(collateralOracle.price.toString()).to.equal(initialPriceCollateral.toString());
    expect(collateralOracle.exponent).to.equal(-8);
    expect(collateralOracle.assetMint.toBase58()).to.equal(collateralMint.toBase58());

    // Initialize Debt Oracle
    const [debtOraclePda] = getOraclePda(marketPda, debtMint);
    const initialPriceDebt = new BN(1_00000000); // $1.00
    const confidenceDebt = new BN(10000); // $0.0001 = 1 bps

    await program.methods
      .initializeMockOracle(initialPriceDebt, -8, confidenceDebt)
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
        assetMint: debtMint,
        oracle: debtOraclePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Verify Market Config oracles were recorded
    const updatedMarket = await program.account.marketConfig.fetch(marketPda);
    expect(updatedMarket.collateralOracle.toBase58()).to.equal(collateralOraclePda.toBase58());
    expect(updatedMarket.debtOracle.toBase58()).to.equal(debtOraclePda.toBase58());
  });

  it("6. Updates mock oracle price and rejects excessive confidence interval", async () => {
    const [collateralOraclePda] = getOraclePda(marketPda, collateralMint);
    const newPrice = new BN(160_00000000); // $160.00
    const validConfidence = new BN(1000000); // $0.01 = 6.25 bps (< 200 bps)

    await program.methods
      .updateMockPrice(newPrice, validConfidence)
      .accountsStrict({
        authority: payer.publicKey,
        oracle: collateralOraclePda,
        updateAuthority: payer.publicKey,
      })
      .rpc();

    const oracle = await program.account.mockOracle.fetch(collateralOraclePda);
    expect(oracle.price.toString()).to.equal(newPrice.toString());

    // Attempt update with confidence > 200 bps (e.g. $10 on $160 is 6.25% = 625 bps)
    const wideConfidence = new BN(10_00000000);
    await assertCustomError(
      program.methods
        .updateMockPrice(newPrice, wideConfidence)
        .accountsStrict({
          authority: payer.publicKey,
          oracle: collateralOraclePda,
          updateAuthority: payer.publicKey,
        })
        .rpc(),
      "OracleConfidenceTooWide"
    );
  });

  it("7. Fails when unauthorized signer attempts to update mock oracle price", async () => {
    const attacker = await createAndFundUser(provider.connection, payer);
    const [collateralOraclePda] = getOraclePda(marketPda, collateralMint);
    const newPrice = new BN(200_00000000);
    const confidence = new BN(100000);

    await assertCustomError(
      program.methods
        .updateMockPrice(newPrice, confidence)
        .accountsStrict({
          authority: attacker.publicKey,
          oracle: collateralOraclePda,
          updateAuthority: attacker.publicKey,
        })
        .signers([attacker])
        .rpc(),
      "UnauthorizedOracleUpdater"
    );
  });

  it("8. Pauses and unpauses market successfully, and blocks unauthorized pause", async () => {
    const attacker = await createAndFundUser(provider.connection, payer);

    // Attacker tries to pause
    await assertCustomError(
      program.methods
        .pauseMarket(true)
        .accountsStrict({
          authority: attacker.publicKey,
          market: marketPda,
        })
        .signers([attacker])
        .rpc(),
      "UnauthorizedMarketAuthority"
    );

    // Legitimate admin pauses
    await program.methods
      .pauseMarket(true)
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
      })
      .rpc();

    let market = await program.account.marketConfig.fetch(marketPda);
    expect(market.isPaused).to.be.true;

    // Legitimate admin unpauses
    await program.methods
      .pauseMarket(false)
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
      })
      .rpc();

    market = await program.account.marketConfig.fetch(marketPda);
    expect(market.isPaused).to.be.false;
  });
});

function anchorRentSysvar(): PublicKey {
  return new PublicKey("SysvarRent111111111111111111111111111111111");
}


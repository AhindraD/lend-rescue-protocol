import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "@anchor-lang/core";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { getProgram, getProvider } from "./setup";
import {
  getMarketPda,
  getCollateralVaultPda,
  getOraclePda,
  getPositionPda,
} from "./helpers/accounts";
import {
  createSyntheticMint,
  createAndFundUser,
  createUserTokenAccount,
  mintTokensTo,
  computeExpectedSyntheticHf,
  refreshOraclePrice,
} from "./helpers/fixtures";
import { assertCustomError, assertPositionState } from "./helpers/assertions";

describe("LendRescue - Position & Lending Operations", () => {
  const provider = getProvider();
  const program = getProgram(provider);
  const payer = (provider.wallet as any).payer as Keypair;

  let collateralMint: PublicKey;
  let debtMint: PublicKey;
  let marketId: BN;
  let marketPda: PublicKey;
  let collateralVaultPda: PublicKey;
  let collateralOraclePda: PublicKey;
  let debtOraclePda: PublicKey;

  let user: Keypair;
  let userCollateralAccount: PublicKey;
  let userDebtAccount: PublicKey;
  let positionId: BN;
  let positionPda: PublicKey;

  before(async () => {
    collateralMint = await createSyntheticMint(provider.connection, payer, 9);
    marketId = new BN(2001);
    [marketPda] = getMarketPda(marketId);
    [collateralVaultPda] = getCollateralVaultPda(marketPda);

    // Debt mint must have market PDA as mint authority so borrowDemoDebt can mint
    debtMint = await createSyntheticMint(provider.connection, payer, 6, marketPda);

    // 1. Initialize Market (80% liquidation threshold = 8000 bps)
    await program.methods
      .initializeMarket(marketId, 8_000, 30)
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
        collateralMint,
        debtMint,
        collateralVault: collateralVaultPda,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: new PublicKey("SysvarRent111111111111111111111111111111111"),
      })
      .rpc();

    // 2. Initialize Oracles ($150 SOL, $1.00 USDC)
    [collateralOraclePda] = getOraclePda(marketPda, collateralMint);
    await program.methods
      .initializeMockOracle(new BN(150_00000000), -8, new BN(500000))
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
        assetMint: collateralMint,
        oracle: collateralOraclePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    [debtOraclePda] = getOraclePda(marketPda, debtMint);
    await program.methods
      .initializeMockOracle(new BN(1_00000000), -8, new BN(10000))
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
        assetMint: debtMint,
        oracle: debtOraclePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // 3. Setup User
    user = await createAndFundUser(provider.connection, payer);
    userCollateralAccount = await createUserTokenAccount(
      provider.connection,
      payer,
      collateralMint,
      user.publicKey
    );
    userDebtAccount = await createUserTokenAccount(
      provider.connection,
      payer,
      debtMint,
      user.publicKey
    );

    // Mint 50 sSOL to user for testing
    await mintTokensTo(
      provider.connection,
      payer,
      collateralMint,
      userCollateralAccount,
      payer,
      50_000000000n
    );

    positionId = new BN(1);
    [positionPda] = getPositionPda(marketPda, user.publicKey, positionId);
  });

  it("1. Creates demo lending position successfully", async () => {
    await program.methods
      .createDemoPosition(positionId)
      .accountsStrict({
        owner: user.publicKey,
        market: marketPda,
        position: positionPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const position = await assertPositionState(program, positionPda, {
      owner: user.publicKey,
      collateralAmount: new BN(0),
      debtAmount: new BN(0),
      status: 0,
    });
    expect(position.market.toBase58()).to.equal(marketPda.toBase58());
  });

  it("2. Deposits collateral into position and updates vault balance", async () => {
    const depositAmount = new BN(10_000000000); // 10 sSOL ($1500 value)

    await program.methods
      .depositCollateral(depositAmount)
      .accountsStrict({
        owner: user.publicKey,
        position: positionPda,
        market: marketPda,
        ownerCollateralAccount: userCollateralAccount,
        collateralVault: collateralVaultPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    await assertPositionState(program, positionPda, {
      collateralAmount: depositAmount,
      debtAmount: new BN(0),
    });
  });

  it("3. Borrows debt within safe health factor bounds", async () => {
    // Refresh oracles to ensure freshness within 60s
    await refreshOraclePrice(program, payer, collateralOraclePda, new BN(150_00000000));
    await refreshOraclePrice(program, payer, debtOraclePda, new BN(1_00000000));

    // 10 sSOL @ $150 = $1,500 collateral. Threshold 80% = $1,200 max safe borrow.
    // Let's borrow 600 sUSDC ($600).
    // Expected HF = ($1500 * 0.8) / $600 = $1200 / $600 = 2.0 = 20,000 (scaled by 10,000).
    const borrowAmount = new BN(600_000000); // 600 sUSDC

    await program.methods
      .borrowDemoDebt(borrowAmount)
      .accountsStrict({
        owner: user.publicKey,
        position: positionPda,
        market: marketPda,
        debtMint,
        collateralOracle: collateralOraclePda,
        debtOracle: debtOraclePda,
        ownerDebtAccount: userDebtAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    await assertPositionState(program, positionPda, {
      collateralAmount: new BN(10_000000000),
      debtAmount: borrowAmount,
    });

    const expectedHf = computeExpectedSyntheticHf({
      collateralAmount: new BN(10_000000000),
      collateralPrice: new BN(150_00000000),
      collateralExp: -8,
      collateralDecimals: 9,
      debtAmount: borrowAmount,
      debtPrice: new BN(1_00000000),
      debtExp: -8,
      debtDecimals: 6,
      liquidationThresholdBps: 8_000,
    });
    expect(expectedHf.toString()).to.equal("20000"); // 2.0
  });

  it("4. Fails to borrow debt exceeding the safe minimum health factor (MIN_SAFE_HF = 1.0)", async () => {
    await refreshOraclePrice(program, payer, collateralOraclePda, new BN(150_00000000));
    await refreshOraclePrice(program, payer, debtOraclePda, new BN(1_00000000));

    // Current debt is 600. Max safe borrow total is 1200.
    // Trying to borrow additional 700 sUSDC would push total debt to 1300 ($1300 > $1200), HF = 1200/1300 = 0.923 < 1.0.
    const excessBorrow = new BN(700_000000);

    await assertCustomError(
      program.methods
        .borrowDemoDebt(excessBorrow)
        .accountsStrict({
          owner: user.publicKey,
          position: positionPda,
          market: marketPda,
          debtMint,
          collateralOracle: collateralOraclePda,
          debtOracle: debtOraclePda,
          ownerDebtAccount: userDebtAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc(),
      "BorrowLimitExceeded"
    );
  });

  it("5. Fails to deposit zero or borrow zero amount", async () => {
    await assertCustomError(
      program.methods
        .depositCollateral(new BN(0))
        .accountsStrict({
          owner: user.publicKey,
          position: positionPda,
          market: marketPda,
          ownerCollateralAccount: userCollateralAccount,
          collateralVault: collateralVaultPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc(),
      "ZeroAmount"
    );

    await assertCustomError(
      program.methods
        .borrowDemoDebt(new BN(0))
        .accountsStrict({
          owner: user.publicKey,
          position: positionPda,
          market: marketPda,
          debtMint,
          collateralOracle: collateralOraclePda,
          debtOracle: debtOraclePda,
          ownerDebtAccount: userDebtAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc(),
      "ZeroAmount"
    );
  });

  it("6. Fails when unauthorized signer attempts to borrow against position", async () => {
    const attacker = await createAndFundUser(provider.connection, payer);
    const attackerDebtAccount = await createUserTokenAccount(
      provider.connection,
      payer,
      debtMint,
      attacker.publicKey
    );

    await assertCustomError(
      program.methods
        .borrowDemoDebt(new BN(100_000000))
        .accountsStrict({
          owner: attacker.publicKey,
          position: positionPda,
          market: marketPda,
          debtMint,
          collateralOracle: collateralOraclePda,
          debtOracle: debtOraclePda,
          ownerDebtAccount: attackerDebtAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([attacker])
        .rpc(),
      "InvalidPositionOwner"
    );
  });

  it("7. Fails to perform operations when market is paused", async () => {
    // Pause market
    await program.methods
      .pauseMarket(true)
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
      })
      .rpc();

    // Deposit fails
    await assertCustomError(
      program.methods
        .depositCollateral(new BN(1_000000000))
        .accountsStrict({
          owner: user.publicKey,
          position: positionPda,
          market: marketPda,
          ownerCollateralAccount: userCollateralAccount,
          collateralVault: collateralVaultPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc(),
      "MarketPaused"
    );

    // Borrow fails
    await assertCustomError(
      program.methods
        .borrowDemoDebt(new BN(10_000000))
        .accountsStrict({
          owner: user.publicKey,
          position: positionPda,
          market: marketPda,
          debtMint,
          collateralOracle: collateralOraclePda,
          debtOracle: debtOraclePda,
          ownerDebtAccount: userDebtAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc(),
      "MarketPaused"
    );

    // Unpause market
    await program.methods
      .pauseMarket(false)
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
      })
      .rpc();
  });
});

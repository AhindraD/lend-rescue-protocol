import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "@anchor-lang/core";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  getProgram,
  getProvider,
  ALL_ACTIONS_MASK,
  ACTION_REPAY_DEBT,
  ACTION_ADD_COLLATERAL,
} from "./setup";
import {
  getMarketPda,
  getCollateralVaultPda,
  getOraclePda,
  getPositionPda,
  getPolicyPda,
  getDemoPoolPda,
  getPoolVaultPda,
  getReceiptPda,
} from "./helpers/accounts";
import {
  createSyntheticMint,
  createAndFundUser,
  createUserTokenAccount,
  mintTokensTo,
  refreshOraclePrice,
} from "./helpers/fixtures";
import { assertCustomError } from "./helpers/assertions";
import { snapshotState, assertSnapshotsEqual } from "./helpers/transactions";

describe("LendRescue - Recovery Security & Account Substitution Defenses", () => {
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

  let demoPoolPda: PublicKey;
  let poolInputVaultPda: PublicKey;
  let poolOutputVaultPda: PublicKey;

  let owner: Keypair;
  let delegate: Keypair;
  let attacker: Keypair;
  let ownerCollateralAccount: PublicKey;
  let ownerDebtAccount: PublicKey;
  let attackerDebtAccount: PublicKey;
  let positionId: BN;
  let positionPda: PublicKey;
  let policyPda: PublicKey;

  before(async () => {
    collateralMint = await createSyntheticMint(provider.connection, payer, 9);
    marketId = new BN(6001);
    [marketPda] = getMarketPda(marketId);
    [collateralVaultPda] = getCollateralVaultPda(marketPda);

    debtMint = await createSyntheticMint(provider.connection, payer, 6, marketPda);

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

    [collateralOraclePda] = getOraclePda(marketPda, collateralMint);
    await program.methods
      .initializeMockOracle(new BN(100_00000000), -8, new BN(500000))
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

    [demoPoolPda] = getDemoPoolPda(marketPda, collateralMint, debtMint);
    [poolInputVaultPda] = getPoolVaultPda(demoPoolPda, collateralMint);
    [poolOutputVaultPda] = getPoolVaultPda(demoPoolPda, debtMint);

    await program.methods
      .initializeDemoSwapPool(30)
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
        inputMint: collateralMint,
        outputMint: debtMint,
        pool: demoPoolPda,
        inputVault: poolInputVaultPda,
        outputVault: poolOutputVaultPda,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: new PublicKey("SysvarRent111111111111111111111111111111111"),
      })
      .rpc();

    owner = await createAndFundUser(provider.connection, payer);
    delegate = await createAndFundUser(provider.connection, payer);
    attacker = await createAndFundUser(provider.connection, payer);

    ownerCollateralAccount = await createUserTokenAccount(
      provider.connection,
      payer,
      collateralMint,
      owner.publicKey
    );
    ownerDebtAccount = await createUserTokenAccount(
      provider.connection,
      payer,
      debtMint,
      owner.publicKey
    );
    attackerDebtAccount = await createUserTokenAccount(
      provider.connection,
      payer,
      debtMint,
      attacker.publicKey
    );

    await mintTokensTo(
      provider.connection,
      payer,
      collateralMint,
      ownerCollateralAccount,
      payer,
      20_000000000n
    );

    positionId = new BN(1);
    [positionPda] = getPositionPda(marketPda, owner.publicKey, positionId);
    [policyPda] = getPolicyPda(positionPda);

    await program.methods
      .createDemoPosition(positionId)
      .accountsStrict({
        owner: owner.publicKey,
        market: marketPda,
        position: positionPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([owner])
      .rpc();

    await program.methods
      .depositCollateral(new BN(10_000000000))
      .accountsStrict({
        owner: owner.publicKey,
        position: positionPda,
        market: marketPda,
        ownerCollateralAccount,
        collateralVault: collateralVaultPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([owner])
      .rpc();

    await refreshOraclePrice(program, payer, collateralOraclePda, new BN(150_00000000));
    await refreshOraclePrice(program, payer, debtOraclePda, new BN(1_00000000));

    await program.methods
      .borrowDemoDebt(new BN(500_000000))
      .accountsStrict({
        owner: owner.publicKey,
        position: positionPda,
        market: marketPda,
        debtMint,
        collateralOracle: collateralOraclePda,
        debtOracle: debtOraclePda,
        ownerDebtAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([owner])
      .rpc();

    const expiry = new BN(Math.floor(Date.now() / 1000) + 86400 * 30);
    await program.methods
      .createRecoveryPolicy(
        delegate.publicKey,
        new BN(500_000000),
        new BN(5_000000000),
        200,
        new BN(11_000),
        ALL_ACTIONS_MASK,
        expiry
      )
      .accountsStrict({
        owner: owner.publicKey,
        position: positionPda,
        policy: policyPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([owner])
      .rpc();
  });

  beforeEach(async () => {
    await refreshOraclePrice(program, payer, collateralOraclePda, new BN(150_00000000));
    await refreshOraclePrice(program, payer, debtOraclePda, new BN(1_00000000));
  });

  it("1. Defends against rogue/fake mock oracle account substitution", async () => {
    // Attacker deploys a rogue mock oracle with a spoofed price of $999,999
    const rogueMarketId = new BN(9999);
    const [rogueMarketPda] = getMarketPda(rogueMarketId);
    const [rogueVaultPda] = getCollateralVaultPda(rogueMarketPda);
    const rogueDebtMint = await createSyntheticMint(provider.connection, payer, 6, rogueMarketPda);

    await program.methods
      .initializeMarket(rogueMarketId, 8_000, 30)
      .accountsStrict({
        authority: payer.publicKey,
        market: rogueMarketPda,
        collateralMint,
        debtMint: rogueDebtMint,
        collateralVault: rogueVaultPda,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: new PublicKey("SysvarRent111111111111111111111111111111111"),
      })
      .rpc();

    const [rogueOraclePda] = getOraclePda(rogueMarketPda, collateralMint);
    await program.methods
      .initializeMockOracle(new BN(999_999_00000000), -8, new BN(10000))
      .accountsStrict({
        authority: payer.publicKey,
        market: rogueMarketPda,
        assetMint: collateralMint,
        oracle: rogueOraclePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Attacker attempts to pass rogueOraclePda as collateral_oracle to execute_recovery
    const nonce = new BN(1);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const args = {
      action: ACTION_REPAY_DEBT,
      inputAmount: new BN(50_000000),
      minOutputAmount: new BN(0),
      nonce,
    };

    await assertCustomError(
      program.methods
        .executeRecovery(args)
        .accountsStrict({
          authority: delegate.publicKey,
          market: marketPda,
          position: positionPda,
          policy: policyPda,
          collateralOracle: rogueOraclePda, // Malicious substitution!
          debtOracle: debtOraclePda,
          collateralVault: collateralVaultPda,
          debtMint,
          userCollateralAccount: ownerCollateralAccount,
          userDebtAccount: ownerDebtAccount,
          demoPool: demoPoolPda,
          poolInputVault: poolInputVaultPda,
          poolOutputVault: poolOutputVaultPda,
          receipt: receiptPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([delegate])
        .rpc(),
      "OracleAssetMismatch"
    );
  });

  it("2. Defends against malicious collateral vault substitution", async () => {
    // Attacker creates a rogue token account and attempts to substitute it as collateral_vault
    const rogueCollateralAccount = await createUserTokenAccount(
      provider.connection,
      payer,
      collateralMint,
      attacker.publicKey
    );

    const nonce = new BN(1);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const args = {
      action: ACTION_ADD_COLLATERAL,
      inputAmount: new BN(1_000000000),
      minOutputAmount: new BN(0),
      nonce,
    };

    await assertCustomError(
      program.methods
        .executeRecovery(args)
        .accountsStrict({
          authority: delegate.publicKey,
          market: marketPda,
          position: positionPda,
          policy: policyPda,
          collateralOracle: collateralOraclePda,
          debtOracle: debtOraclePda,
          collateralVault: rogueCollateralAccount, // Rogue vault!
          debtMint,
          userCollateralAccount: ownerCollateralAccount,
          userDebtAccount: ownerDebtAccount,
          demoPool: demoPoolPda,
          poolInputVault: poolInputVaultPda,
          poolOutputVault: poolOutputVaultPda,
          receipt: receiptPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([delegate])
        .rpc(),
      "InvalidMint"
    );
  });

  it("3. Defends against unauthorized user token account draining (UnauthorizedCaller)", async () => {
    // Attacker passes attacker's debt account while signing as delegate
    // But attacker is not the position owner nor the authorized caller
    const rogueVictim = await createAndFundUser(provider.connection, payer);
    const victimDebtAccount = await createUserTokenAccount(
      provider.connection,
      payer,
      debtMint,
      rogueVictim.publicKey
    );

    const nonce = new BN(1);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const args = {
      action: ACTION_REPAY_DEBT,
      inputAmount: new BN(10_000000),
      minOutputAmount: new BN(0),
      nonce,
    };

    await assertCustomError(
      program.methods
        .executeRecovery(args)
        .accountsStrict({
          authority: delegate.publicKey,
          market: marketPda,
          position: positionPda,
          policy: policyPda,
          collateralOracle: collateralOraclePda,
          debtOracle: debtOraclePda,
          collateralVault: collateralVaultPda,
          debtMint,
          userCollateralAccount: ownerCollateralAccount,
          userDebtAccount: victimDebtAccount, // Victim account not owned by owner or caller!
          demoPool: demoPoolPda,
          poolInputVault: poolInputVaultPda,
          poolOutputVault: poolOutputVaultPda,
          receipt: receiptPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([delegate])
        .rpc(),
      "UnauthorizedCaller"
    );
  });

  it("4. Defends against execution when market is paused", async () => {
    // Pause market
    await program.methods
      .pauseMarket(true)
      .accountsStrict({
        authority: payer.publicKey,
        market: marketPda,
      })
      .rpc();

    const nonce = new BN(1);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const args = {
      action: ACTION_REPAY_DEBT,
      inputAmount: new BN(50_000000),
      minOutputAmount: new BN(0),
      nonce,
    };

    await assertCustomError(
      program.methods
        .executeRecovery(args)
        .accountsStrict({
          authority: delegate.publicKey,
          market: marketPda,
          position: positionPda,
          policy: policyPda,
          collateralOracle: collateralOraclePda,
          debtOracle: debtOraclePda,
          collateralVault: collateralVaultPda,
          debtMint,
          userCollateralAccount: ownerCollateralAccount,
          userDebtAccount: ownerDebtAccount,
          demoPool: demoPoolPda,
          poolInputVault: poolInputVaultPda,
          poolOutputVault: poolOutputVaultPda,
          receipt: receiptPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([delegate])
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

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
  ACTION_SWAP_AND_REPAY,
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
  transferTokens,
  approveTokenDelegate,
  getTokenBalance,
  computeExpectedSyntheticHf,
  computeExpectedSwapOutput,
  refreshOraclePrice,
} from "./helpers/fixtures";
import {
  assertPositionState,
  assertPolicyState,
  assertReceiptData,
  assertCustomError,
} from "./helpers/assertions";

describe("LendRescue - Complete Recovery Happy-Path Lifecycle", () => {
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
  let ownerCollateralAccount: PublicKey;
  let ownerDebtAccount: PublicKey;
  let positionId: BN;
  let positionPda: PublicKey;
  let policyPda: PublicKey;

  before(async () => {
    // 1. Synthetic Assets
    collateralMint = await createSyntheticMint(provider.connection, payer, 9);
    marketId = new BN(4001);
    [marketPda] = getMarketPda(marketId);
    [collateralVaultPda] = getCollateralVaultPda(marketPda);

    debtMint = await createSyntheticMint(provider.connection, payer, 6, marketPda);

    // 2. Initialize Market (80% liquidation threshold, 30 bps swap fee)
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

    // 3. Initialize Oracles: sSOL @ $150.00, sUSDC @ $1.00
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

    // 4. Initialize Demo Swap Pool (sSOL -> sUSDC)
    [demoPoolPda] = getDemoPoolPda(marketPda, collateralMint, debtMint);
    [poolInputVaultPda] = getPoolVaultPda(demoPoolPda, collateralMint);
    [poolOutputVaultPda] = getPoolVaultPda(demoPoolPda, debtMint);

    await program.methods
      .initializeDemoSwapPool(30) // 0.30% fee
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

    // Fund Demo Swap Pool liquidity: 100 sSOL input reserve, 15,000 sUSDC output reserve
    const poolSeedFunder = await createAndFundUser(provider.connection, payer);
    const funderCollateral = await createUserTokenAccount(
      provider.connection,
      payer,
      collateralMint,
      poolSeedFunder.publicKey
    );
    await mintTokensTo(
      provider.connection,
      payer,
      collateralMint,
      funderCollateral,
      payer,
      100_000000000n
    );
    // Transfer 100 sSOL to pool input vault
    const { transfer } = await import("@solana/spl-token");
    await transfer(
      provider.connection,
      poolSeedFunder,
      funderCollateral,
      poolInputVaultPda,
      poolSeedFunder,
      100_000000000n
    );

    // Mint 15,000 sUSDC to pool output vault directly via market PDA mint authority
    // To do this simply, we borrow from a temporary position or mint directly using market PDA
    // Since payer created debtMint, wait: debtMint authority was set to marketPda.
    // Let's create a temporary market position with poolSeedFunder, deposit 200 sSOL, and borrow 15,000 sUSDC into output vault!
    const tempPosId = new BN(999);
    const [tempPosPda] = getPositionPda(marketPda, poolSeedFunder.publicKey, tempPosId);
    await program.methods
      .createDemoPosition(tempPosId)
      .accountsStrict({
        owner: poolSeedFunder.publicKey,
        market: marketPda,
        position: tempPosPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([poolSeedFunder])
      .rpc();

    const tempCollateralAcc = await createUserTokenAccount(
      provider.connection,
      payer,
      collateralMint,
      poolSeedFunder.publicKey
    );
    await mintTokensTo(
      provider.connection,
      payer,
      collateralMint,
      tempCollateralAcc,
      payer,
      200_000000000n
    );

    await program.methods
      .depositCollateral(new BN(200_000000000))
      .accountsStrict({
        owner: poolSeedFunder.publicKey,
        position: tempPosPda,
        market: marketPda,
        ownerCollateralAccount: tempCollateralAcc,
        collateralVault: collateralVaultPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([poolSeedFunder])
      .rpc();

    const tempDebtAcc = await createUserTokenAccount(
      provider.connection,
      payer,
      debtMint,
      poolSeedFunder.publicKey
    );

    await refreshOraclePrice(program, payer, collateralOraclePda, new BN(150_00000000));
    await refreshOraclePrice(program, payer, debtOraclePda, new BN(1_00000000));

    // Borrow 15,000 sUSDC into tempDebtAcc
    await program.methods
      .borrowDemoDebt(new BN(15_000_000000))
      .accountsStrict({
        owner: poolSeedFunder.publicKey,
        position: tempPosPda,
        market: marketPda,
        debtMint,
        collateralOracle: collateralOraclePda,
        debtOracle: debtOraclePda,
        ownerDebtAccount: tempDebtAcc,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([poolSeedFunder])
      .rpc();

    // Transfer 15,000 sUSDC to poolOutputVaultPda
    await transferTokens(
      payer,
      tempDebtAcc,
      poolOutputVaultPda,
      poolSeedFunder,
      15_000_000000n
    );

    // 5. Setup Owner & Delegate
    owner = await createAndFundUser(provider.connection, payer);
    delegate = await createAndFundUser(provider.connection, payer);

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

    // Mint 20 sSOL to owner
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
  });

  it("1. Creates position, deposits 10 sSOL ($1500), and borrows 800 sUSDC ($800)", async () => {
    // Create position
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

    // Deposit 10 sSOL
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

    // Borrow 800 sUSDC
    await refreshOraclePrice(program, payer, collateralOraclePda, new BN(150_00000000));
    await refreshOraclePrice(program, payer, debtOraclePda, new BN(1_00000000));

    await program.methods
      .borrowDemoDebt(new BN(800_000000))
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

    await assertPositionState(program, positionPda, {
      collateralAmount: new BN(10_000000000),
      debtAmount: new BN(800_000000),
    });

    const hfBefore = computeExpectedSyntheticHf({
      collateralAmount: new BN(10_000000000),
      collateralPrice: new BN(150_00000000),
      collateralExp: -8,
      collateralDecimals: 9,
      debtAmount: new BN(800_000000),
      debtPrice: new BN(1_00000000),
      debtExp: -8,
      debtDecimals: 6,
      liquidationThresholdBps: 8_000,
    });
    expect(hfBefore.toString()).to.equal("15000"); // 1.5000
  });

  it("2. Creates recovery policy with authorized delegate", async () => {
    const expiry = new BN(Math.floor(Date.now() / 1000) + 86400 * 30);
    const maxDebtRepayment = new BN(500_000000); // 500 sUSDC
    const maxCollateralSpend = new BN(5_000000000); // 5 sSOL
    const maxSlippageBps = 200; // 2%
    const minResultingHf = new BN(11_000); // 1.1000
    const allowedActions = ALL_ACTIONS_MASK; // 7

    await program.methods
      .createRecoveryPolicy(
        delegate.publicKey,
        maxDebtRepayment,
        maxCollateralSpend,
        maxSlippageBps,
        minResultingHf,
        allowedActions,
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

    await assertPolicyState(program, policyPda, {
      owner: owner.publicKey,
      authorizedDelegate: delegate.publicKey,
      isActive: true,
      nonce: new BN(1),
    });

    // Owner approves delegate on SPL token accounts up to policy limits
    await approveTokenDelegate(payer, ownerDebtAccount, delegate.publicKey, owner, 500_000000n);
    await approveTokenDelegate(payer, ownerCollateralAccount, delegate.publicKey, owner, 5_000000000n);
  });

  it("3. Price shock: sSOL drops from $150 to $90 -> Health Factor drops to 0.9000 (Danger)", async () => {
    await program.methods
      .updateMockPrice(new BN(90_00000000), new BN(100000))
      .accountsStrict({
        authority: payer.publicKey,
        oracle: collateralOraclePda,
        updateAuthority: payer.publicKey,
      })
      .rpc();

    // 10 sSOL @ $90 = $900. 80% threshold = $720. Debt = $800.
    // HF = 720 / 800 = 0.9000 = 9000 (unhealthy!)
    const currentHf = computeExpectedSyntheticHf({
      collateralAmount: new BN(10_000000000),
      collateralPrice: new BN(90_00000000),
      collateralExp: -8,
      collateralDecimals: 9,
      debtAmount: new BN(800_000000),
      debtPrice: new BN(1_00000000),
      debtExp: -8,
      debtDecimals: 6,
      liquidationThresholdBps: 8_000,
    });
    expect(currentHf.toString()).to.equal("9000"); // 0.9000
  });

  it("4. Recovery Action 1 (RepayDebt): Delegate repays 200 sUSDC, restores HF to 1.2000, creates Receipt", async () => {
    await refreshOraclePrice(program, payer, collateralOraclePda, new BN(90_00000000), new BN(100000));
    await refreshOraclePrice(program, payer, debtOraclePda, new BN(1_00000000), new BN(10000));

    // Delegate triggers recovery: repays 200 sUSDC from ownerDebtAccount
    const repayAmount = new BN(200_000000);
    const nonce = new BN(1);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const args = {
      action: ACTION_REPAY_DEBT,
      inputAmount: repayAmount,
      minOutputAmount: new BN(0),
      nonce,
    };

    await program.methods
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
      .rpc();

    // Verify position debt reduced from 800 down to 600 sUSDC
    await assertPositionState(program, positionPda, {
      collateralAmount: new BN(10_000000000),
      debtAmount: new BN(600_000000),
    });

    // Post-HF: $720 / $600 = 1.2000 = 12,000
    const postHf = computeExpectedSyntheticHf({
      collateralAmount: new BN(10_000000000),
      collateralPrice: new BN(90_00000000),
      collateralExp: -8,
      collateralDecimals: 9,
      debtAmount: new BN(600_000000),
      debtPrice: new BN(1_00000000),
      debtExp: -8,
      debtDecimals: 6,
      liquidationThresholdBps: 8_000,
    });
    expect(postHf.toString()).to.equal("12000");

    // Verify policy nonce consumed and incremented to 2
    await assertPolicyState(program, policyPda, {
      nonce: new BN(2),
    });

    // Verify on-chain RecoveryReceipt
    await assertReceiptData(program, receiptPda, {
      position: positionPda,
      owner: owner.publicKey,
      delegate: delegate.publicKey,
      actionExecuted: ACTION_REPAY_DEBT,
      inputAmount: repayAmount,
      repaymentAmount: repayAmount,
      nonceConsumed: nonce,
    });
  });

  it("5. Recovery Action 2 (AddCollateral): Delegate deposits 2 sSOL collateral, raises HF to 1.4400", async () => {
    await refreshOraclePrice(program, payer, collateralOraclePda, new BN(90_00000000), new BN(100000));
    await refreshOraclePrice(program, payer, debtOraclePda, new BN(1_00000000), new BN(10000));

    // Delegate posts 2 sSOL additional collateral from owner's account
    const addCollateralAmount = new BN(2_000000000); // 2 sSOL
    const nonce = new BN(2);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const args = {
      action: ACTION_ADD_COLLATERAL,
      inputAmount: addCollateralAmount,
      minOutputAmount: new BN(0),
      nonce,
    };

    await program.methods
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
      .rpc();

    // Collateral increased from 10 to 12 sSOL
    await assertPositionState(program, positionPda, {
      collateralAmount: new BN(12_000000000),
      debtAmount: new BN(600_000000),
    });

    // Post-HF: (12 * $90 * 0.8) / 600 = $864 / $600 = 1.4400 = 14,400
    const postHf = computeExpectedSyntheticHf({
      collateralAmount: new BN(12_000000000),
      collateralPrice: new BN(90_00000000),
      collateralExp: -8,
      collateralDecimals: 9,
      debtAmount: new BN(600_000000),
      debtPrice: new BN(1_00000000),
      debtExp: -8,
      debtDecimals: 6,
      liquidationThresholdBps: 8_000,
    });
    expect(postHf.toString()).to.equal("14400");

    // Verify policy nonce consumed and incremented to 3
    await assertPolicyState(program, policyPda, {
      nonce: new BN(3),
    });

    // Verify on-chain RecoveryReceipt
    await assertReceiptData(program, receiptPda, {
      position: positionPda,
      owner: owner.publicKey,
      delegate: delegate.publicKey,
      actionExecuted: ACTION_ADD_COLLATERAL,
      inputAmount: addCollateralAmount,
      repaymentAmount: new BN(0),
      nonceConsumed: nonce,
    });
  });

  it("6. Recovery Action 3 (SwapAndRepay): Sells 1 sSOL from collateral vault via DemoSwapPool to repay debt", async () => {
    await refreshOraclePrice(program, payer, collateralOraclePda, new BN(90_00000000), new BN(100000));
    await refreshOraclePrice(program, payer, debtOraclePda, new BN(1_00000000), new BN(10000));

    // Current reserves in pool:
    const reserveInBefore = await getTokenBalance(provider.connection, poolInputVaultPda);
    const reserveOutBefore = await getTokenBalance(provider.connection, poolOutputVaultPda);

    const collateralToSpend = new BN(1_000000000); // 1 sSOL
    const { amountOut: expectedAmountOut } = computeExpectedSwapOutput(
      reserveInBefore,
      reserveOutBefore,
      collateralToSpend,
      30
    );

    const nonce = new BN(3);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const minOutput = expectedAmountOut.mul(new BN(98)).div(new BN(100)); // 2% slippage tolerance

    const args = {
      action: ACTION_SWAP_AND_REPAY,
      inputAmount: collateralToSpend,
      minOutputAmount: minOutput,
      nonce,
    };

    await program.methods
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
      .rpc();

    // Position collateral: 12 - 1 = 11 sSOL
    // Position debt: 600 - expectedAmountOut
    const expectedDebtRemaining = new BN(600_000000).sub(expectedAmountOut);
    await assertPositionState(program, positionPda, {
      collateralAmount: new BN(11_000000000),
      debtAmount: expectedDebtRemaining,
    });

    // Policy nonce incremented to 4
    await assertPolicyState(program, policyPda, {
      nonce: new BN(4),
    });

    // Verify on-chain RecoveryReceipt
    await assertReceiptData(program, receiptPda, {
      position: positionPda,
      owner: owner.publicKey,
      delegate: delegate.publicKey,
      actionExecuted: ACTION_SWAP_AND_REPAY,
      inputAmount: collateralToSpend,
      repaymentAmount: expectedAmountOut,
      nonceConsumed: nonce,
    });
  });

  it("7. Revokes policy and verifies it becomes permanently unusable", async () => {
    await program.methods
      .revokeRecoveryPolicy()
      .accountsStrict({
        owner: owner.publicKey,
        policy: policyPda,
      })
      .signers([owner])
      .rpc();

    await assertPolicyState(program, policyPda, {
      isActive: false,
      nonce: new BN(5),
    });

    // Trying to execute recovery on revoked policy fails with PolicyRevoked
    const [receiptPda] = getReceiptPda(positionPda, new BN(5));
    const args = {
      action: ACTION_REPAY_DEBT,
      inputAmount: new BN(10_000000),
      minOutputAmount: new BN(0),
      nonce: new BN(5),
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
      "PolicyRevoked"
    );
  });
});

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
  refreshOraclePrice,
} from "./helpers/fixtures";
import { assertCustomError } from "./helpers/assertions";
import { snapshotState, assertSnapshotsEqual } from "./helpers/transactions";

describe("LendRescue - Recovery Failure Paths & Adversarial Scenarios", () => {
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
  let positionId: BN;
  let positionPda: PublicKey;
  let policyPda: PublicKey;

  before(async () => {
    collateralMint = await createSyntheticMint(provider.connection, payer, 9);
    marketId = new BN(5001);
    [marketPda] = getMarketPda(marketId);
    [collateralVaultPda] = getCollateralVaultPda(marketPda);

    debtMint = await createSyntheticMint(provider.connection, payer, 6, marketPda);

    // Initialize Market
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

    // Initialize Oracles ($100 collateral, $1 debt)
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

    // Initialize Demo Swap Pool
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

    // Setup Owner, Delegate, and Attacker
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

    // Mint collateral to owner
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

    // Create position, deposit 10 sSOL ($1,000 value), borrow 600 sUSDC
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
      .borrowDemoDebt(new BN(600_000000))
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

    // Create policy with maxDebtRepayment = 200 sUSDC, maxCollateralSpend = 2 sSOL, minResultingHf = 1.2 (12,000)
    const expiry = new BN(Math.floor(Date.now() / 1000) + 86400 * 30);
    await program.methods
      .createRecoveryPolicy(
        delegate.publicKey,
        new BN(200_000000), // maxDebtRepayment = 200 sUSDC
        new BN(2_000000000), // maxCollateralSpend = 2 sSOL
        100, // 1% max slippage
        new BN(12_000), // min resulting HF = 1.2
        ACTION_REPAY_DEBT | ACTION_ADD_COLLATERAL, // note: SWAP not allowed in this policy!
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

    // Owner approves delegate for the policy limits on SPL token accounts
    await approveTokenDelegate(payer, ownerDebtAccount, delegate.publicKey, owner, 200_000000n);
    await approveTokenDelegate(payer, ownerCollateralAccount, delegate.publicKey, owner, 2_000000000n);
  });

  beforeEach(async () => {
    await refreshOraclePrice(program, payer, collateralOraclePda, new BN(150_00000000));
    await refreshOraclePrice(program, payer, debtOraclePda, new BN(1_00000000));
  });

  it("1. Rejects unauthorized recovery delegate and verifies atomic rollback", async () => {
    const before = await snapshotState(
      provider.connection,
      program,
      [ownerCollateralAccount, ownerDebtAccount, collateralVaultPda],
      positionPda,
      policyPda
    );

    const nonce = new BN(0);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const args = {
      action: ACTION_REPAY_DEBT,
      inputAmount: new BN(100_000000),
      minOutputAmount: new BN(0),
      nonce,
    };

    await assertCustomError(
      program.methods
        .executeRecovery(args)
        .accountsStrict({
          authority: attacker.publicKey,
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
        .signers([attacker])
        .rpc(),
      "UnauthorizedCaller"
    );

    const after = await snapshotState(
      provider.connection,
      program,
      [ownerCollateralAccount, ownerDebtAccount, collateralVaultPda],
      positionPda,
      policyPda
    );
    assertSnapshotsEqual(before, after);

    // Verify receipt was NOT created
    const receiptInfo = await provider.connection.getAccountInfo(receiptPda);
    expect(receiptInfo).to.be.null;
  });

  it("2. Rejects repayment exceeding policy maximum limit (ExceedsMaxDebtRepayment)", async () => {
    const before = await snapshotState(
      provider.connection,
      program,
      [ownerDebtAccount],
      positionPda,
      policyPda
    );

    const nonce = new BN(0);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    // Policy max is 200 sUSDC. Trying to repay 250 sUSDC must fail.
    const args = {
      action: ACTION_REPAY_DEBT,
      inputAmount: new BN(250_000000),
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
      "ExceedsMaxDebtRepayment"
    );

    const after = await snapshotState(
      provider.connection,
      program,
      [ownerDebtAccount],
      positionPda,
      policyPda
    );
    assertSnapshotsEqual(before, after);
  });

  it("3. Rejects collateral spend exceeding policy maximum limit (ExceedsMaxCollateralSpend)", async () => {
    const before = await snapshotState(
      provider.connection,
      program,
      [ownerCollateralAccount, collateralVaultPda],
      positionPda,
      policyPda
    );

    const nonce = new BN(0);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    // Policy max collateral spend is 2 sSOL. Trying to spend 3 sSOL must fail.
    const args = {
      action: ACTION_ADD_COLLATERAL,
      inputAmount: new BN(3_000000000),
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
      "ExceedsMaxCollateralSpend"
    );

    const after = await snapshotState(
      provider.connection,
      program,
      [ownerCollateralAccount, collateralVaultPda],
      positionPda,
      policyPda
    );
    assertSnapshotsEqual(before, after);
  });

  it("4. Rejects action not permitted by policy bitset (ActionNotAllowed)", async () => {
    const nonce = new BN(0);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    // SWAP_AND_REPAY was omitted from this policy's allowed_actions
    const args = {
      action: ACTION_SWAP_AND_REPAY,
      inputAmount: new BN(1_000000000),
      minOutputAmount: new BN(10_000000),
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
      "ActionNotAllowed"
    );
  });

  it("5. Rejects recovery if resulting health factor fails to achieve policy minimum (ResultingHealthFactorTooLow)", async () => {
    // Drop price to $50. Collateral = 10 * $50 = $500. 80% = $400. Debt = 600.
    // HF = 400 / 600 = 0.6666 = 6,666.
    // If we repay only 10 sUSDC: new debt = 590. New HF = 400 / 590 = 0.6779 = 6,779.
    // Policy requires minResultingHf = 12,000 (1.2). Must reject!
    await program.methods
      .updateMockPrice(new BN(50_00000000), new BN(100000))
      .accountsStrict({
        authority: payer.publicKey,
        oracle: collateralOraclePda,
        updateAuthority: payer.publicKey,
      })
      .rpc();

    const before = await snapshotState(
      provider.connection,
      program,
      [ownerDebtAccount],
      positionPda,
      policyPda
    );

    const nonce = new BN(0);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const args = {
      action: ACTION_REPAY_DEBT,
      inputAmount: new BN(10_000000), // Tiny repayment fails to reach target HF
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
      "ResultingHealthFactorTooLow"
    );

    const after = await snapshotState(
      provider.connection,
      program,
      [ownerDebtAccount],
      positionPda,
      policyPda
    );
    assertSnapshotsEqual(before, after);

    // Restore price back to $100
    await program.methods
      .updateMockPrice(new BN(100_00000000), new BN(100000))
      .accountsStrict({
        authority: payer.publicKey,
        oracle: collateralOraclePda,
        updateAuthority: payer.publicKey,
      })
      .rpc();
  });

  it("6. Rejects replayed recovery nonce (InvalidNonce)", async () => {
    // First execute a valid recovery with nonce 0 to advance nonce to 1
    const nonce = new BN(0);
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const validArgs = {
      action: ACTION_REPAY_DEBT,
      inputAmount: new BN(150_000000), // Repay 150 sUSDC (debt -> 450, HF -> 800/450 = 1.77 > 1.2)
      minOutputAmount: new BN(0),
      nonce,
    };

    await program.methods
      .executeRecovery(validArgs)
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

    // Now try to replay nonce 0 again!
    // Case A: Attacker supplies receipt matching current policy nonce (1) but replays args.nonce = 0
    const [receiptForCurrentNonce] = getReceiptPda(positionPda, new BN(1));
    await assertCustomError(
      program.methods
        .executeRecovery(validArgs) // validArgs.nonce is 0, but on-chain nonce is 1
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
          receipt: receiptForCurrentNonce,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([delegate])
        .rpc(),
      "InvalidNonce"
    );

    // Case B: Attacker tries to pass the old receipt (nonce 0) -> blocked by PDA seeds constraint
    const [replayReceiptPda] = getReceiptPda(positionPda, nonce);
    await assertCustomError(
      program.methods
        .executeRecovery(validArgs)
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
          receipt: replayReceiptPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([delegate])
        .rpc(),
      "ConstraintSeeds"
    );
  });

  it("7. Rejects swap-and-repay when output slippage exceeds minimum required amount (SlippageExceeded)", async () => {
    // 1. Seed pool with reserves: 2 sSOL and 200 sUSDC
    await transferTokens(payer, ownerCollateralAccount, poolInputVaultPda, owner, 2_000000000n);
    await transferTokens(payer, ownerDebtAccount, poolOutputVaultPda, owner, 200_000000n);

    // 2. Owner updates policy to enable SWAP_AND_REPAY
    const newExpiry = new BN(Math.floor(Date.now() / 1000) + 86400 * 30);
    await program.methods
      .updateRecoveryPolicy(
        delegate.publicKey,
        new BN(500_000000), // 500 sUSDC
        new BN(5_000000000), // 5 sSOL
        500, // 5% slippage limit
        new BN(10_000), // min HF = 1.0
        ALL_ACTIONS_MASK,
        newExpiry
      )
      .accountsStrict({
        owner: owner.publicKey,
        policy: policyPda,
      })
      .signers([owner])
      .rpc();

    // 3. Delegate attempts swap with an impossible minOutputAmount (200 sUSDC when pool output is ~66 sUSDC)
    const policyAcc = await program.account.recoveryPolicy.fetch(policyPda);
    const nonce = policyAcc.executionNonce;
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const before = await snapshotState(
      provider.connection,
      program,
      [ownerCollateralAccount, ownerDebtAccount, collateralVaultPda],
      positionPda,
      policyPda
    );

    const excessiveSlippageArgs = {
      action: ACTION_SWAP_AND_REPAY,
      inputAmount: new BN(1_000000000), // 1 sSOL
      minOutputAmount: new BN(200_000000), // 200 sUSDC (expected ~66 sUSDC)
      nonce,
    };

    await assertCustomError(
      program.methods
        .executeRecovery(excessiveSlippageArgs)
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
      "SlippageExceeded"
    );

    const after = await snapshotState(
      provider.connection,
      program,
      [ownerCollateralAccount, ownerDebtAccount, collateralVaultPda],
      positionPda,
      policyPda
    );
    assertSnapshotsEqual(before, after);
  });

  it("8. Rejects recovery execution when policy has been revoked (PolicyRevoked)", async () => {
    // Revoke the policy (increments nonce, isActive = false)
    await program.methods
      .revokeRecoveryPolicy()
      .accountsStrict({
        owner: owner.publicKey,
        policy: policyPda,
      })
      .signers([owner])
      .rpc();

    const policyAcc = await program.account.recoveryPolicy.fetch(policyPda);
    const nonce = policyAcc.executionNonce;
    const [receiptPda] = getReceiptPda(positionPda, nonce);

    const before = await snapshotState(
      provider.connection,
      program,
      [ownerCollateralAccount, ownerDebtAccount, collateralVaultPda],
      positionPda,
      policyPda
    );

    const recoveryArgs = {
      action: ACTION_REPAY_DEBT,
      inputAmount: new BN(50_000000),
      minOutputAmount: new BN(0),
      nonce,
    };

    await assertCustomError(
      program.methods
        .executeRecovery(recoveryArgs)
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

    const after = await snapshotState(
      provider.connection,
      program,
      [ownerCollateralAccount, ownerDebtAccount, collateralVaultPda],
      positionPda,
      policyPda
    );
    assertSnapshotsEqual(before, after);
  });
});

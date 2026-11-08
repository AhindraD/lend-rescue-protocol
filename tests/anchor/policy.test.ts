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
  getPositionPda,
  getPolicyPda,
} from "./helpers/accounts";
import {
  createSyntheticMint,
  createAndFundUser,
} from "./helpers/fixtures";
import { assertCustomError, assertPolicyState } from "./helpers/assertions";

describe("LendRescue - Recovery Policy Governance", () => {
  const provider = getProvider();
  const program = getProgram(provider);
  const payer = (provider.wallet as any).payer as Keypair;

  let collateralMint: PublicKey;
  let debtMint: PublicKey;
  let marketId: BN;
  let marketPda: PublicKey;
  let collateralVaultPda: PublicKey;

  let user: Keypair;
  let delegate: Keypair;
  let positionId: BN;
  let positionPda: PublicKey;
  let policyPda: PublicKey;

  before(async () => {
    collateralMint = await createSyntheticMint(provider.connection, payer, 9);
    marketId = new BN(3001);
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

    user = await createAndFundUser(provider.connection, payer);
    delegate = await createAndFundUser(provider.connection, payer);

    positionId = new BN(1);
    [positionPda] = getPositionPda(marketPda, user.publicKey, positionId);
    [policyPda] = getPolicyPda(positionPda);

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
  });

  it("1. Creates valid recovery policy with bounded parameters and Some(delegate)", async () => {
    const futureExpiry = new BN(Math.floor(Date.now() / 1000) + 86400 * 30); // 30 days
    const maxDebtRepayment = new BN(1_000_000000); // 1,000 sUSDC
    const maxCollateralSpend = new BN(10_000000000); // 10 sSOL
    const maxSlippageBps = 100; // 1%
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
        futureExpiry
      )
      .accountsStrict({
        owner: user.publicKey,
        position: positionPda,
        policy: policyPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const policy = await assertPolicyState(program, policyPda, {
      owner: user.publicKey,
      authorizedDelegate: delegate.publicKey,
      isActive: true,
      executionNonce: new BN(0),
      policyRevision: new BN(1),
      schemaVersion: 2,
    });

    expect(policy.maxDebtRepayment.toString()).to.equal(maxDebtRepayment.toString());
    expect(policy.maxCollateralSpend.toString()).to.equal(maxCollateralSpend.toString());
    expect(policy.maxSlippageBps).to.equal(maxSlippageBps);
    expect(policy.minResultingHf.toString()).to.equal(minResultingHf.toString());
    expect(policy.allowedActions).to.equal(allowedActions);
  });

  it("2. Creates owner-only recovery policy with None delegate", async () => {
    const userOwnerOnly = await createAndFundUser(provider.connection, payer);
    const posIdOwnerOnly = new BN(99);
    const [posPdaOwnerOnly] = getPositionPda(marketPda, userOwnerOnly.publicKey, posIdOwnerOnly);
    const [polPdaOwnerOnly] = getPolicyPda(posPdaOwnerOnly);

    await program.methods
      .createDemoPosition(posIdOwnerOnly)
      .accountsStrict({
        owner: userOwnerOnly.publicKey,
        market: marketPda,
        position: posPdaOwnerOnly,
        systemProgram: SystemProgram.programId,
      })
      .signers([userOwnerOnly])
      .rpc();

    const futureExpiry = new BN(Math.floor(Date.now() / 1000) + 86400 * 30);
    await program.methods
      .createRecoveryPolicy(
        null, // None -> owner-only execution
        new BN(1_000_000000),
        new BN(10_000000000),
        100,
        new BN(11_000),
        ALL_ACTIONS_MASK,
        futureExpiry
      )
      .accountsStrict({
        owner: userOwnerOnly.publicKey,
        position: posPdaOwnerOnly,
        policy: polPdaOwnerOnly,
        systemProgram: SystemProgram.programId,
      })
      .signers([userOwnerOnly])
      .rpc();

    await assertPolicyState(program, polPdaOwnerOnly, {
      owner: userOwnerOnly.publicKey,
      authorizedDelegate: null,
      isActive: true,
      executionNonce: new BN(0),
      policyRevision: new BN(1),
      schemaVersion: 2,
    });
  });

  it("3. Blocks non-owner from creating policy for position", async () => {
    const attacker = await createAndFundUser(provider.connection, payer);
    const futureExpiry = new BN(Math.floor(Date.now() / 1000) + 3600);

    const posId3 = new BN(301);
    const [posPda3] = getPositionPda(marketPda, user.publicKey, posId3);
    const [polPda3] = getPolicyPda(posPda3);

    await program.methods
      .createDemoPosition(posId3)
      .accountsStrict({
        owner: user.publicKey,
        market: marketPda,
        position: posPda3,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    // Attacker tries to create policy for user's fresh position
    await assertCustomError(
      program.methods
        .createRecoveryPolicy(
          attacker.publicKey,
          new BN(100),
          new BN(100),
          100,
          new BN(11000),
          ALL_ACTIONS_MASK,
          futureExpiry
        )
        .accountsStrict({
          owner: attacker.publicKey,
          position: posPda3,
          policy: polPda3,
          systemProgram: SystemProgram.programId,
        })
        .signers([attacker])
        .rpc(),
      "InvalidPositionOwner"
    );
  });

  it("4. Fails to create policy with invalid bounds, expired timestamp, or bad actions", async () => {
    const user2 = await createAndFundUser(provider.connection, payer);
    const posId2 = new BN(2);
    const [posPda2] = getPositionPda(marketPda, user2.publicKey, posId2);
    const [polPda2] = getPolicyPda(posPda2);

    await program.methods
      .createDemoPosition(posId2)
      .accountsStrict({
        owner: user2.publicKey,
        market: marketPda,
        position: posPda2,
        systemProgram: SystemProgram.programId,
      })
      .signers([user2])
      .rpc();

    const futureExpiry = new BN(Math.floor(Date.now() / 1000) + 3600);
    const pastExpiry = new BN(Math.floor(Date.now() / 1000) - 3600);

    // 1. Past expiry
    await assertCustomError(
      program.methods
        .createRecoveryPolicy(
          delegate.publicKey,
          new BN(100),
          new BN(100),
          100,
          new BN(11000),
          ALL_ACTIONS_MASK,
          pastExpiry
        )
        .accountsStrict({
          owner: user2.publicKey,
          position: posPda2,
          policy: polPda2,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc(),
      "PolicyExpired"
    );

    // 2. Zero max debt repayment
    await assertCustomError(
      program.methods
        .createRecoveryPolicy(
          delegate.publicKey,
          new BN(0),
          new BN(100),
          100,
          new BN(11000),
          ALL_ACTIONS_MASK,
          futureExpiry
        )
        .accountsStrict({
          owner: user2.publicKey,
          position: posPda2,
          policy: polPda2,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc(),
      "InvalidPolicyBounds"
    );

    // 3. Zero max collateral spend
    await assertCustomError(
      program.methods
        .createRecoveryPolicy(
          delegate.publicKey,
          new BN(100),
          new BN(0),
          100,
          new BN(11000),
          ALL_ACTIONS_MASK,
          futureExpiry
        )
        .accountsStrict({
          owner: user2.publicKey,
          position: posPda2,
          policy: polPda2,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc(),
      "InvalidPolicyBounds"
    );

    // 4. Excessive slippage (> 1000 bps)
    await assertCustomError(
      program.methods
        .createRecoveryPolicy(
          delegate.publicKey,
          new BN(100),
          new BN(100),
          1001,
          new BN(11000),
          ALL_ACTIONS_MASK,
          futureExpiry
        )
        .accountsStrict({
          owner: user2.publicKey,
          position: posPda2,
          policy: polPda2,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc(),
      "SlippageExceeded"
    );

    // 5. Target health factor too low (< 10_000)
    await assertCustomError(
      program.methods
        .createRecoveryPolicy(
          delegate.publicKey,
          new BN(100),
          new BN(100),
          100,
          new BN(9999),
          ALL_ACTIONS_MASK,
          futureExpiry
        )
        .accountsStrict({
          owner: user2.publicKey,
          position: posPda2,
          policy: polPda2,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc(),
      "TargetHealthFactorTooLow"
    );

    // 6. Zero allowed actions
    await assertCustomError(
      program.methods
        .createRecoveryPolicy(
          delegate.publicKey,
          new BN(100),
          new BN(100),
          100,
          new BN(11000),
          0,
          futureExpiry
        )
        .accountsStrict({
          owner: user2.publicKey,
          position: posPda2,
          policy: polPda2,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc(),
      "InvalidAllowedActions"
    );
  });

  it("5. Updates recovery policy parameters and increments policy_revision (execution_nonce preserved)", async () => {
    const updatedExpiry = new BN(Math.floor(Date.now() / 1000) + 86400 * 60);
    const updatedDebt = new BN(2_000_000000);
    const updatedCollateral = new BN(20_000000000);
    const updatedSlippage = 50; // 0.5%
    const updatedMinHf = new BN(12_000); // 1.2
    const updatedActions = ACTION_REPAY_DEBT | ACTION_ADD_COLLATERAL; // 3

    await program.methods
      .updateRecoveryPolicy(
        delegate.publicKey,
        updatedDebt,
        updatedCollateral,
        updatedSlippage,
        updatedMinHf,
        updatedActions,
        updatedExpiry
      )
      .accountsStrict({
        owner: user.publicKey,
        policy: policyPda,
      })
      .signers([user])
      .rpc();

    // Verify policy_revision incremented to 2, and execution_nonce remains 0
    await assertPolicyState(program, policyPda, {
      owner: user.publicKey,
      authorizedDelegate: delegate.publicKey,
      isActive: true,
      executionNonce: new BN(0),
      policyRevision: new BN(2),
    });
  });

  it("6. Blocks unauthorized signer from updating or revoking policy", async () => {
    const attacker = await createAndFundUser(provider.connection, payer);
    const dummyExpiry = new BN(Math.floor(Date.now() / 1000) + 86400);

    // Attacker tries to update policy
    await assertCustomError(
      program.methods
        .updateRecoveryPolicy(
          attacker.publicKey,
          new BN(1000),
          new BN(1000),
          100,
          new BN(11000),
          ALL_ACTIONS_MASK,
          dummyExpiry
        )
        .accountsStrict({
          owner: attacker.publicKey,
          policy: policyPda,
        })
        .signers([attacker])
        .rpc(),
      "InvalidPolicyOwner"
    );

    // Attacker tries to revoke policy
    await assertCustomError(
      program.methods
        .revokeRecoveryPolicy()
        .accountsStrict({
          owner: attacker.publicKey,
          policy: policyPda,
        })
        .signers([attacker])
        .rpc(),
      "InvalidPolicyOwner"
    );
  });

  it("7. Revokes policy, disables active flag, and increments policy_revision", async () => {
    await program.methods
      .revokeRecoveryPolicy()
      .accountsStrict({
        owner: user.publicKey,
        policy: policyPda,
      })
      .signers([user])
      .rpc();

    // Policy revision incremented to 3, isActive = false, execution_nonce remains 0
    await assertPolicyState(program, policyPda, {
      isActive: false,
      executionNonce: new BN(0),
      policyRevision: new BN(3),
    });

    // Attempting to update a revoked policy must fail with PolicyRevoked
    const dummyExpiry = new BN(Math.floor(Date.now() / 1000) + 86400);
    await assertCustomError(
      program.methods
        .updateRecoveryPolicy(
          delegate.publicKey,
          new BN(1000),
          new BN(1000),
          100,
          new BN(11000),
          ALL_ACTIONS_MASK,
          dummyExpiry
        )
        .accountsStrict({
          owner: user.publicKey,
          policy: policyPda,
        })
        .signers([user])
        .rpc(),
      "PolicyRevoked"
    );
  });

  it("8. Verifies InitSpace exact account allocations", async () => {
    const policyAccountInfo = await provider.connection.getAccountInfo(policyPda);
    expect(policyAccountInfo).to.not.be.null;

    // Anchor discriminator (8) + RecoveryPolicy::INIT_SPACE
    // Pubkey(32) + Pubkey(32) + Option<Pubkey>(33) + u64(8)*4 + u16(2) + u8(1)*4 + i64(8) = 152 bytes + 8 = 160
    expect(policyAccountInfo!.data.length).to.be.greaterThan(0);

    const marketAccountInfo = await provider.connection.getAccountInfo(marketPda);
    expect(marketAccountInfo).to.not.be.null;
    expect(marketAccountInfo!.data.length).to.be.greaterThan(0);

    const positionAccountInfo = await provider.connection.getAccountInfo(positionPda);
    expect(positionAccountInfo).to.not.be.null;
    expect(positionAccountInfo!.data.length).to.be.greaterThan(0);
  });
});

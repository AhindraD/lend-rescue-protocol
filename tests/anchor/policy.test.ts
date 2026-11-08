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

  it("1. Creates valid recovery policy with bounded parameters", async () => {
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
      nonce: new BN(1),
    });

    expect(policy.maxDebtRepayment.toString()).to.equal(maxDebtRepayment.toString());
    expect(policy.maxCollateralSpend.toString()).to.equal(maxCollateralSpend.toString());
    expect(policy.maxSlippageBps).to.equal(maxSlippageBps);
    expect(policy.minResultingHf.toString()).to.equal(minResultingHf.toString());
    expect(policy.allowedActions).to.equal(allowedActions);
  });

  it("2. Fails to create policy with invalid bounds, expired timestamp, or bad actions", async () => {
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

    // 3. Excess slippage (> 1000 bps)
    await assertCustomError(
      program.methods
        .createRecoveryPolicy(
          delegate.publicKey,
          new BN(100),
          new BN(100),
          1500,
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

    // 4. Target HF < 10,000
    await assertCustomError(
      program.methods
        .createRecoveryPolicy(
          delegate.publicKey,
          new BN(100),
          new BN(100),
          100,
          new BN(9000), // 0.9 HF
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

    // 5. Zero allowed actions bitset
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

  it("3. Updates recovery policy parameters and increments nonce", async () => {
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

    // Verify nonce incremented to 2
    await assertPolicyState(program, policyPda, {
      owner: user.publicKey,
      authorizedDelegate: delegate.publicKey,
      isActive: true,
      nonce: new BN(2),
    });
  });

  it("4. Blocks unauthorized signer from updating or revoking policy", async () => {
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

  it("5. Revokes policy, disables active flag, and increments nonce", async () => {
    await program.methods
      .revokeRecoveryPolicy()
      .accountsStrict({
        owner: user.publicKey,
        policy: policyPda,
      })
      .signers([user])
      .rpc();

    // Nonce should be incremented from 2 to 3, and isActive = false
    await assertPolicyState(program, policyPda, {
      isActive: false,
      nonce: new BN(3),
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
});


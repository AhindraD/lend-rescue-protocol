import { expect } from "chai";
import { PublicKey } from "@solana/web3.js";
import { BN, Program } from "@anchor-lang/core";
import { LendRescue } from "../../../target/types/lend_rescue";

export async function assertCustomError(
  actionPromise: Promise<any>,
  expectedErrorName: string
): Promise<void> {
  try {
    await actionPromise;
    expect.fail(`Expected transaction to fail with error '${expectedErrorName}', but it succeeded.`);
  } catch (err: any) {
    const errorStr = (err.message || "") + (err.logs ? " " + err.logs.join(" ") : "") + (err.toString() || "");
    const errorMatched =
      errorStr.includes(expectedErrorName) ||
      (err.error && err.error.errorCode && err.error.errorCode.code === expectedErrorName);

    if (!errorMatched) {
      // Print detailed error info for debugging
      console.error("Unexpected error caught:", err);
    }
    expect(
      errorMatched,
      `Expected error '${expectedErrorName}', but got: ${errorStr}`
    ).to.be.true;
  }
}

export async function assertPositionState(
  program: Program<LendRescue>,
  positionPubkey: PublicKey,
  expected: {
    owner?: PublicKey;
    collateralAmount?: BN;
    debtAmount?: BN;
    status?: number;
  }
): Promise<any> {
  const position = await program.account.position.fetch(positionPubkey);
  if (expected.owner) {
    expect(position.owner.toBase58()).to.equal(expected.owner.toBase58());
  }
  if (expected.collateralAmount !== undefined) {
    expect(position.collateralAmount.toString()).to.equal(
      expected.collateralAmount.toString()
    );
  }
  if (expected.debtAmount !== undefined) {
    expect(position.debtAmount.toString()).to.equal(
      expected.debtAmount.toString()
    );
  }
  if (expected.status !== undefined) {
    expect(position.status).to.equal(expected.status);
  }
  return position;
}

export async function assertPolicyState(
  program: Program<LendRescue>,
  policyPubkey: PublicKey,
  expected: {
    owner?: PublicKey;
    authorizedDelegate?: PublicKey | null;
    isActive?: boolean;
    nonce?: BN | number;
    executionNonce?: BN | number;
    policyRevision?: BN | number;
    schemaVersion?: number;
  }
): Promise<any> {
  const policy = await program.account.recoveryPolicy.fetch(policyPubkey);
  if (expected.owner) {
    expect(policy.owner.toBase58()).to.equal(expected.owner.toBase58());
  }
  if (expected.authorizedDelegate !== undefined) {
    if (expected.authorizedDelegate === null) {
      expect(policy.authorizedDelegate).to.be.null;
    } else {
      expect(policy.authorizedDelegate).to.not.be.null;
      expect(policy.authorizedDelegate!.toBase58()).to.equal(
        expected.authorizedDelegate.toBase58()
      );
    }
  }
  if (expected.isActive !== undefined) {
    expect(policy.isActive).to.equal(expected.isActive);
  }
  if (expected.executionNonce !== undefined) {
    expect(policy.executionNonce.toString()).to.equal(
      expected.executionNonce.toString()
    );
  } else if (expected.nonce !== undefined) {
    expect(policy.executionNonce.toString()).to.equal(
      expected.nonce.toString()
    );
  }
  if (expected.policyRevision !== undefined) {
    expect(policy.policyRevision.toString()).to.equal(
      expected.policyRevision.toString()
    );
  }
  if (expected.schemaVersion !== undefined) {
    expect(policy.schemaVersion).to.equal(expected.schemaVersion);
  }
  return policy;
}

export async function assertReceiptData(
  program: Program<LendRescue>,
  receiptPubkey: PublicKey,
  expected: {
    position: PublicKey;
    owner: PublicKey;
    delegate: PublicKey;
    actionExecuted: number;
    inputAmount: BN;
    repaymentAmount: BN;
    nonceConsumed: BN | number;
  }
): Promise<any> {
  const receipt = await program.account.recoveryReceipt.fetch(receiptPubkey);
  expect(receipt.position.toBase58()).to.equal(expected.position.toBase58());
  expect(receipt.owner.toBase58()).to.equal(expected.owner.toBase58());
  expect(receipt.delegate.toBase58()).to.equal(expected.delegate.toBase58());
  expect(receipt.actionExecuted).to.equal(expected.actionExecuted);
  expect(receipt.inputAmount.toString()).to.equal(expected.inputAmount.toString());
  expect(receipt.repaymentAmount.toString()).to.equal(
    expected.repaymentAmount.toString()
  );
  const expectedNonce = BN.isBN(expected.nonceConsumed)
    ? expected.nonceConsumed.toString()
    : expected.nonceConsumed.toString();
  expect(receipt.nonceConsumed.toString()).to.equal(expectedNonce);
  expect(receipt.status).to.equal(0);
  return receipt;
}


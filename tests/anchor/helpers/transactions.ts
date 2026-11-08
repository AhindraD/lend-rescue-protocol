import { Connection, PublicKey } from "@solana/web3.js";
import { Program, BN } from "@anchor-lang/core";
import { expect } from "chai";
import { LendRescue } from "../../../target/types/lend_rescue";
import { getTokenBalance } from "./fixtures";

export interface AccountSnapshot {
  tokenBalances: Map<string, string>;
  positionCollateral?: string;
  positionDebt?: string;
  policyNonce?: string;
  policyIsActive?: boolean;
}

export async function snapshotState(
  connection: Connection,
  program: Program<LendRescue>,
  tokenAccounts: PublicKey[],
  positionPubkey?: PublicKey,
  policyPubkey?: PublicKey
): Promise<AccountSnapshot> {
  const tokenBalances = new Map<string, string>();
  for (const acc of tokenAccounts) {
    try {
      const balance = await getTokenBalance(connection, acc);
      tokenBalances.set(acc.toBase58(), balance.toString());
    } catch {
      tokenBalances.set(acc.toBase58(), "0");
    }
  }

  let positionCollateral: string | undefined;
  let positionDebt: string | undefined;
  if (positionPubkey) {
    try {
      const pos = await program.account.position.fetch(positionPubkey);
      positionCollateral = pos.collateralAmount.toString();
      positionDebt = pos.debtAmount.toString();
    } catch {}
  }

  let policyNonce: string | undefined;
  let policyIsActive: boolean | undefined;
  if (policyPubkey) {
    try {
      const pol = await program.account.recoveryPolicy.fetch(policyPubkey);
      policyNonce = pol.nonce.toString();
      policyIsActive = pol.isActive;
    } catch {}
  }

  return {
    tokenBalances,
    positionCollateral,
    positionDebt,
    policyNonce,
    policyIsActive,
  };
}

export function assertSnapshotsEqual(
  before: AccountSnapshot,
  after: AccountSnapshot
): void {
  for (const [key, val] of before.tokenBalances.entries()) {
    expect(
      after.tokenBalances.get(key),
      `Token balance for ${key} changed across failed transaction!`
    ).to.equal(val);
  }

  if (before.positionCollateral !== undefined) {
    expect(
      after.positionCollateral,
      "Position collateral changed across failed transaction!"
    ).to.equal(before.positionCollateral);
  }

  if (before.positionDebt !== undefined) {
    expect(
      after.positionDebt,
      "Position debt changed across failed transaction!"
    ).to.equal(before.positionDebt);
  }

  if (before.policyNonce !== undefined) {
    expect(
      after.policyNonce,
      "Policy nonce changed across failed transaction!"
    ).to.equal(before.policyNonce);
  }

  if (before.policyIsActive !== undefined) {
    expect(
      after.policyIsActive,
      "Policy active state changed across failed transaction!"
    ).to.equal(before.policyIsActive);
  }
}

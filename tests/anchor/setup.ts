import * as anchor from "@anchor-lang/core";
import { Program, BN, AnchorProvider } from "@anchor-lang/core";
import { Connection, Keypair, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { LendRescue } from "../../target/types/lend_rescue";
import idl from "../../target/idl/lend_rescue.json";

export const PROGRAM_ID = new PublicKey("FDH6cF9xvoTBpJiPgAEciVcNueEqhLYxXQcPDpM6PpUV");

export const MARKET_SEED = Buffer.from("market");
export const ORACLE_SEED = Buffer.from("oracle");
export const POSITION_SEED = Buffer.from("position");
export const POLICY_SEED = Buffer.from("policy");
export const DEMO_POOL_SEED = Buffer.from("demo_pool");
export const RECEIPT_SEED = Buffer.from("receipt");
export const COLLATERAL_VAULT_SEED = Buffer.from("collateral_vault");
export const POOL_VAULT_SEED = Buffer.from("pool_vault");

export const ACTION_REPAY_DEBT = 1;
export const ACTION_ADD_COLLATERAL = 2;
export const ACTION_SWAP_AND_REPAY = 4;
export const ALL_ACTIONS_MASK = ACTION_REPAY_DEBT | ACTION_ADD_COLLATERAL | ACTION_SWAP_AND_REPAY;

export const BPS_DIVISOR = new BN(10_000);
export const HF_PRECISION = new BN(10_000);
export const MIN_SAFE_HF = new BN(10_000);
export const MAX_CONFIDENCE_BPS = new BN(200);
export const MAX_PERMITTED_SLIPPAGE_BPS = 1_000;
export const MAX_ORACLE_AGE_SECS = 60;

export function getProvider(): AnchorProvider {
  return AnchorProvider.env();
}

export function getProgram(provider?: AnchorProvider): Program<LendRescue> {
  const p = provider || getProvider();
  return new Program<LendRescue>(idl as any, p);
}


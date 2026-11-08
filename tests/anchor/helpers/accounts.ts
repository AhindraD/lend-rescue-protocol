import { PublicKey } from "@solana/web3.js";
import { BN } from "@anchor-lang/core";
import {
  PROGRAM_ID,
  MARKET_SEED,
  ORACLE_SEED,
  POSITION_SEED,
  POLICY_SEED,
  DEMO_POOL_SEED,
  RECEIPT_SEED,
  COLLATERAL_VAULT_SEED,
  POOL_VAULT_SEED,
} from "../setup";

export function toLeBytesU64(value: BN | number): Buffer {
  const bn = BN.isBN(value) ? value : new BN(value);
  return bn.toArrayLike(Buffer, "le", 8);
}

export function getMarketPda(
  marketId: BN | number,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [MARKET_SEED, toLeBytesU64(marketId)],
    programId
  );
}

export function getCollateralVaultPda(
  market: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [COLLATERAL_VAULT_SEED, market.toBuffer()],
    programId
  );
}

export function getOraclePda(
  market: PublicKey,
  assetMint: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [ORACLE_SEED, market.toBuffer(), assetMint.toBuffer()],
    programId
  );
}

export function getPositionPda(
  market: PublicKey,
  owner: PublicKey,
  positionId: BN | number,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [POSITION_SEED, market.toBuffer(), owner.toBuffer(), toLeBytesU64(positionId)],
    programId
  );
}

export function getPolicyPda(
  position: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [POLICY_SEED, position.toBuffer()],
    programId
  );
}

export function getDemoPoolPda(
  market: PublicKey,
  inputMint: PublicKey,
  outputMint: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [DEMO_POOL_SEED, market.toBuffer(), inputMint.toBuffer(), outputMint.toBuffer()],
    programId
  );
}

export function getPoolVaultPda(
  pool: PublicKey,
  mint: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [POOL_VAULT_SEED, pool.toBuffer(), mint.toBuffer()],
    programId
  );
}

export function getReceiptPda(
  position: PublicKey,
  nonce: BN | number,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [RECEIPT_SEED, position.toBuffer(), toLeBytesU64(nonce)],
    programId
  );
}


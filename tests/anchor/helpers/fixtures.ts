import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  getMinimumBalanceForRentExemptMint,
  MINT_SIZE,
  ACCOUNT_SIZE,
  createInitializeMint2Instruction,
  createInitializeAccountInstruction,
  createMintToInstruction,
  createTransferInstruction,
  createApproveInstruction,
  getAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { BN, Program } from "@anchor-lang/core";
import { LendRescue } from "../../../target/types/lend_rescue";
import { BPS_DIVISOR, getProvider } from "../setup";

export const CONFIRM_OPTS = {
  skipPreflight: true,
  commitment: "processed" as const,
  preflightCommitment: "processed" as const,
};

export async function fundAccount(
  connection: Connection,
  payer: Keypair,
  recipient: PublicKey,
  lamports: number = 2 * LAMPORTS_PER_SOL
): Promise<void> {
  const provider = getProvider();
  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: recipient,
      lamports,
    })
  );
  const signers: Keypair[] = [];
  if (!payer.publicKey.equals(provider.wallet.publicKey)) {
    signers.push(payer);
  }
  await provider.sendAndConfirm(tx, signers);
}

export async function createSyntheticMint(
  connection: Connection,
  payer: Keypair,
  decimals: number,
  mintAuthority: PublicKey = payer.publicKey
): Promise<PublicKey> {
  const provider = getProvider();
  const mintKeypair = Keypair.generate();
  const rent = await getMinimumBalanceForRentExemptMint(provider.connection);
  const tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: mintKeypair.publicKey,
      space: MINT_SIZE,
      lamports: rent,
      programId: TOKEN_PROGRAM_ID,
    }),
    createInitializeMint2Instruction(
      mintKeypair.publicKey,
      decimals,
      mintAuthority,
      null,
      TOKEN_PROGRAM_ID
    )
  );
  const signers: Keypair[] = [mintKeypair];
  if (!payer.publicKey.equals(provider.wallet.publicKey)) {
    signers.push(payer);
  }
  await provider.sendAndConfirm(tx, signers);
  return mintKeypair.publicKey;
}

export async function createAndFundUser(
  connection: Connection,
  payer: Keypair,
  lamports: number = 5 * LAMPORTS_PER_SOL
): Promise<Keypair> {
  const user = Keypair.generate();
  await fundAccount(connection, payer, user.publicKey, lamports);
  return user;
}

export async function createUserTokenAccount(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey
): Promise<PublicKey> {
  const provider = getProvider();
  const tokenAccountKeypair = Keypair.generate();
  const rent = await provider.connection.getMinimumBalanceForRentExemption(ACCOUNT_SIZE);
  const tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: tokenAccountKeypair.publicKey,
      space: ACCOUNT_SIZE,
      lamports: rent,
      programId: TOKEN_PROGRAM_ID,
    }),
    createInitializeAccountInstruction(
      tokenAccountKeypair.publicKey,
      mint,
      owner,
      TOKEN_PROGRAM_ID
    )
  );
  const signers: Keypair[] = [tokenAccountKeypair];
  if (!payer.publicKey.equals(provider.wallet.publicKey)) {
    signers.push(payer);
  }
  await provider.sendAndConfirm(tx, signers);
  return tokenAccountKeypair.publicKey;
}

export async function mintTokensTo(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  destination: PublicKey,
  authority: Keypair,
  amount: number | bigint
): Promise<string> {
  const provider = getProvider();
  const tx = new Transaction().add(
    createMintToInstruction(
      mint,
      destination,
      authority.publicKey,
      amount,
      [],
      TOKEN_PROGRAM_ID
    )
  );
  const signers: Keypair[] = [];
  if (!payer.publicKey.equals(provider.wallet.publicKey)) {
    signers.push(payer);
  }
  if (!authority.publicKey.equals(provider.wallet.publicKey)) {
    signers.push(authority);
  }
  return await provider.sendAndConfirm(tx, signers);
}

export async function transferTokens(
  payer: Keypair,
  source: PublicKey,
  destination: PublicKey,
  owner: Keypair,
  amount: number | bigint
): Promise<string> {
  const provider = getProvider();
  const tx = new Transaction().add(
    createTransferInstruction(
      source,
      destination,
      owner.publicKey,
      amount,
      [],
      TOKEN_PROGRAM_ID
    )
  );
  const signers: Keypair[] = [];
  if (!payer.publicKey.equals(provider.wallet.publicKey)) {
    signers.push(payer);
  }
  if (!owner.publicKey.equals(provider.wallet.publicKey)) {
    signers.push(owner);
  }
  return await provider.sendAndConfirm(tx, signers);
}

export async function approveTokenDelegate(
  payer: Keypair,
  account: PublicKey,
  delegate: PublicKey,
  owner: Keypair,
  amount: number | bigint
): Promise<string> {
  const provider = getProvider();
  const tx = new Transaction().add(
    createApproveInstruction(
      account,
      delegate,
      owner.publicKey,
      amount,
      [],
      TOKEN_PROGRAM_ID
    )
  );
  const signers: Keypair[] = [];
  if (!payer.publicKey.equals(provider.wallet.publicKey)) {
    signers.push(payer);
  }
  if (!owner.publicKey.equals(provider.wallet.publicKey)) {
    signers.push(owner);
  }
  return await provider.sendAndConfirm(tx, signers);
}

export async function refreshOraclePrice(
  program: Program<LendRescue>,
  payer: Keypair,
  oraclePda: PublicKey,
  price: BN = new BN(150_00000000),
  confidence: BN = new BN(500000)
): Promise<void> {
  await program.methods
    .updateMockPrice(price, confidence)
    .accountsStrict({
      authority: payer.publicKey,
      oracle: oraclePda,
      updateAuthority: payer.publicKey,
    })
    .rpc();
}

export async function getTokenBalance(
  connection: Connection,
  tokenAccount: PublicKey
): Promise<BN> {
  const acc = await getAccount(connection, tokenAccount);
  return new BN(acc.amount.toString());
}

/**
 * Independent reference calculation for synthetic health factor (scaled by 10,000)
 */
export function computeExpectedSyntheticHf(params: {
  collateralAmount: BN;
  collateralPrice: BN;
  collateralExp: number;
  collateralDecimals: number;
  debtAmount: BN;
  debtPrice: BN;
  debtExp: number;
  debtDecimals: number;
  liquidationThresholdBps: number;
}): BN {
  const {
    collateralAmount,
    collateralPrice,
    collateralExp,
    collateralDecimals,
    debtAmount,
    debtPrice,
    debtExp,
    debtDecimals,
    liquidationThresholdBps,
  } = params;

  if (debtAmount.isZero()) {
    return new BN("18446744073709551615"); // u64::MAX
  }
  if (collateralAmount.isZero() || collateralPrice.lten(0)) {
    return new BN(0);
  }

  const sC = collateralExp - collateralDecimals;
  const sD = debtExp - debtDecimals;

  let num = collateralAmount
    .mul(collateralPrice)
    .mul(new BN(liquidationThresholdBps));
  let den = debtAmount.mul(debtPrice);

  if (sC > sD) {
    const factor = new BN(10).pow(new BN(sC - sD));
    num = num.mul(factor);
  } else if (sD > sC) {
    const factor = new BN(10).pow(new BN(sD - sC));
    den = den.mul(factor);
  }

  return num.div(den);
}

/**
 * Independent reference calculation for constant-product swap output & fee
 */
export function computeExpectedSwapOutput(
  reserveIn: BN,
  reserveOut: BN,
  amountIn: BN,
  feeBps: number
): { amountOut: BN; feeAmount: BN } {
  const feeMultiplier = BPS_DIVISOR.sub(new BN(feeBps));
  const feeAmount = amountIn.mul(new BN(feeBps)).div(BPS_DIVISOR);

  const amountInWithFee = amountIn.mul(feeMultiplier);
  const numerator = amountInWithFee.mul(reserveOut);
  const denominator = reserveIn.mul(BPS_DIVISOR).add(amountInWithFee);

  const amountOut = numerator.div(denominator);
  return { amountOut, feeAmount };
}

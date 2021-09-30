import { Connection, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import bs58 from "bs58";

const DEVNET = true;
// differences between devnet and testnet
// testnet is super rate limited
// pyth btc price data is 1 less decimal place
// usd, yes, and no token mints are different
// program id and pda are different

export const connection = new Connection(
  DEVNET ? "https://api.devnet.solana.com" : "https://api.testnet.solana.com",
  "singleGossip"
);
export const NULL_PUBLIC_KEY = new PublicKey(new Uint8Array(Array(32).fill(0)));

export const BTC_PRICE_ACCOUNT = new PublicKey(
  DEVNET
    ? "HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J"
    : "DJW6f4ZVqCnpYNN9rNuzqUcCvkVtBgixo8mq9FKSsCbJ"
);
export const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: PublicKey = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);

// REPLACE_ME --------------------
export const ALICE_KEYPAIR = Keypair.fromSecretKey(new Uint8Array([]));
export const BOB_KEYPAIR = Keypair.fromSecretKey(new Uint8Array([]));
export const USD_TOKEN_MINT = new PublicKey(DEVNET ? "" : "");
export const BETTING_MARKET_PROGRAM_ID = new PublicKey(DEVNET ? "" : "");
export const PDA = new PublicKey(DEVNET ? "" : "");
export const BETTING_MARKET_DATA_ACCOUNT = new PublicKey(DEVNET ? "" : "");
// -------------------------------

export function keypairFromNums(nums: string) {
  return Keypair.fromSecretKey(
    Uint8Array.from(nums.split(",").map((s) => parseInt(s)))
  );
}

export async function findPda() {
  const [pda, _bump_seed] = await PublicKey.findProgramAddress(
    [Buffer.from("betting")],
    BETTING_MARKET_PROGRAM_ID
  );
  console.log("pda", pda.toBase58());
  return pda;
}

export async function findAssociatedTokenAddress(
  walletAddress: PublicKey,
  tokenMintAddress: PublicKey
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        walletAddress.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        tokenMintAddress.toBuffer(),
      ],
      SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
    )
  )[0];
}

export async function getTokenAccountBalanceString(
  tokenAccountPubkey: PublicKey
) {
  const tokenAccountBalanceString =
    (await getTokenAccountBalance(tokenAccountPubkey))?.uiAmountString ||
    "unitialized";
  return tokenAccountBalanceString;
}

export async function getTokenAccountBalance(tokenAccountPubkey: PublicKey) {
  const accountOwner = (
    await connection.getParsedAccountInfo(tokenAccountPubkey)
  ).value?.owner;
  if (accountOwner?.toBase58() === TOKEN_PROGRAM_ID.toBase58()) {
    return await getTokenAccountBalanceUnsafe(tokenAccountPubkey);
  }
  return null;
}

export async function getTokenAccountBalanceUnsafe(
  tokenAccountPubkey: PublicKey
) {
  const response = await connection.getTokenAccountBalance(tokenAccountPubkey);
  return response.value;
}

export const createAssociatedTokenAccount = async (
  userAccountKeypair: Keypair,
  tokenMintPubkey: PublicKey
) => {
  const associatedTokenAddress = await findAssociatedTokenAddress(
    userAccountKeypair.publicKey,
    tokenMintPubkey
  );
  const ix = Token.createAssociatedTokenAccountInstruction(
    SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    tokenMintPubkey,
    associatedTokenAddress,
    userAccountKeypair.publicKey,
    userAccountKeypair.publicKey
  );
  const tx = new Transaction().add(ix);
  await connection.sendTransaction(tx, [userAccountKeypair], {
    skipPreflight: false,
    preflightCommitment: "singleGossip",
  });

  console.log("finished createAssociatedTokenAccount transaction");
  console.log("transaction", bs58.encode(tx.signature!));
};

export const DEVMODE = false;

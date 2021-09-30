import * as BufferLayout from "@solana/buffer-layout";
import { PublicKey } from "@solana/web3.js";

/**
 * Layout for a public key
 */
const publicKey = (property = "publicKey") => {
  return BufferLayout.blob(32, property);
};

/**
 * Layout for a 64bit unsigned value
 */
const uint64 = (property = "uint64") => {
  return BufferLayout.blob(8, property);
};

export const BETTING_MARKET_DATA_LAYOUT = BufferLayout.struct([
  BufferLayout.u8("isInitialized"),
  BufferLayout.u8("result"),
  publicKey("yesTokenMint"),
  publicKey("noTokenMint"),
  publicKey("usdTokenAccount"),
  uint64("strikePrice"),
  publicKey("judge"),
  BufferLayout.blob(1000 - 138),
  BufferLayout.blob(808, "buyAmountsForYesPrice"),
  BufferLayout.blob(2000 - 1808),
  BufferLayout.blob(808, "buyAmountsForNoPrice"),
  BufferLayout.blob(10000 - 2808),
  BufferLayout.blob(42320 - 10000, "userAccountsForPrice"),
  BufferLayout.blob(50000 - 42320),
  BufferLayout.blob(58080 - 50000, "payoutInUsdForPrice"),
  BufferLayout.blob(60000 - 58080),
  BufferLayout.blob(68080 - 60000, "payoutAmountsForPrice"),
  BufferLayout.blob(70000 - 68080),
  BufferLayout.blob(73200 - 70000, "payoutUserAccounts"),
  BufferLayout.blob(80000 - 73200),
  BufferLayout.blob(83200 - 80000, "payoutMints"),
  BufferLayout.blob(90000 - 83200),
  BufferLayout.blob(90800 - 90000, "payoutAmounts"),
]);

export interface RawBettingMarketData {
  isInitialized: number;
  result: number;
  yesTokenMint: Uint8Array;
  noTokenMint: Uint8Array;
  usdTokenAccount: Uint8Array;
  strikePrice: number;
  judge: Uint8Array;
  buyAmountsForYesPrice: Uint8Array;
  buyAmountsForNoPrice: Uint8Array;
  // userAccountsForPrice: Uint8Array;
  // payoutInUsdForPrice: Uint8Array;
  // payoutAmountsForPrice: Uint8Array;
  payoutUserAccounts: Uint8Array;
  payoutMints: Uint8Array;
  payoutAmounts: Uint8Array;
}

export interface BettingMarketState {
  isInitialized: boolean;
  result: number;
  yesTokenMint: PublicKey;
  noTokenMint: PublicKey;
  usdTokenAccount: PublicKey;
  strikePrice: number;
  judge: PublicKey;
  buyAmountsForYesPrice: number[];
  buyAmountsForNoPrice: number[];
  // userAccountsForPrice: PublicKey[];
  // payoutInUsdForPrice: number[];
  // payoutAmountsForPrice: number[];
  payoutUserAccounts: PublicKey[];
  payoutMints: PublicKey[];
  payoutAmounts: number[];
}

export function formatBettingMarketInfo(
  bettingMarketState: BettingMarketState
) {
  const bettingMarketStateFormatted = {
    isInitialized: bettingMarketState.isInitialized,
    result: bettingMarketState.result,
    yesTokenMint: bettingMarketState.yesTokenMint.toBase58(),
    noTokenMint: bettingMarketState.noTokenMint.toBase58(),
    usdTokenAccount: bettingMarketState.usdTokenAccount.toBase58(),
    strikePrice: bettingMarketState.strikePrice,
    judge: bettingMarketState.judge.toBase58(),
  };
  return bettingMarketStateFormatted;
}

import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import BN from "bn.js";
import {
  BETTING_MARKET_DATA_LAYOUT,
  RawBettingMarketData,
  BettingMarketState,
} from "./layout";
import {
  connection,
  USD_TOKEN_MINT,
  BETTING_MARKET_PROGRAM_ID,
  PDA,
  BETTING_MARKET_DATA_ACCOUNT,
  BTC_PRICE_ACCOUNT,
} from "./common";
import bs58 from "bs58";
import { parsePriceData } from "@pythnetwork/client";

export const getBtcPriceData = async () => {
  return getPriceData(BTC_PRICE_ACCOUNT);
};

export const getPriceData = async (priceAccountPubkey: PublicKey) => {
  const accountInfo = await connection.getAccountInfo(priceAccountPubkey);
  const { price, confidence, validSlot } = parsePriceData(accountInfo!.data);
  // console.log(
  //   `BTC price data: $${price} \xB1$${confidence} at slot ${validSlot}`
  // );
  return [price, confidence, validSlot];
};

export const initBettingMarket = async (
  initializerAccountKeypair: Keypair,
  bettingMarketProgramId: PublicKey
) => {
  const judgeAccountPubkey = new PublicKey(
    "uJ1Vu2YAAaR7pdX1FEQ9Mi9zQen2J6guXUSKRuxBBYQ"
  );
  initBettingMarketWithParams(
    initializerAccountKeypair,
    new Keypair(),
    USD_TOKEN_MINT,
    new Keypair(),
    new Keypair(),
    new Keypair(),
    judgeAccountPubkey,
    bettingMarketProgramId
  );
};

export const initBettingMarketWithParams = async (
  initializerAccountKeypair: Keypair,
  bettingMarketDataAccountKeypair: Keypair,
  usdTokenMintPubkey: PublicKey,
  yesTokenMintAccountKeypair: Keypair,
  noTokenMintAccountKeypair: Keypair,
  usdTokenAccountKeypair: Keypair,
  judgeAccountPubkey: PublicKey,
  bettingMarketProgramId: PublicKey
) => {
  const useExistingBettingMarketDataAccount = false;
  let bettingMarketDataAccountPubkey = BETTING_MARKET_DATA_ACCOUNT;
  if (!useExistingBettingMarketDataAccount) {
    const createBettingMarketDataAccountIx = SystemProgram.createAccount({
      space: 100000,
      lamports: await connection.getMinimumBalanceForRentExemption(
        100000,
        "singleGossip"
      ),
      fromPubkey: initializerAccountKeypair.publicKey,
      newAccountPubkey: bettingMarketDataAccountKeypair.publicKey,
      programId: bettingMarketProgramId,
    });
    const tx2 = new Transaction().add(createBettingMarketDataAccountIx);
    await connection.sendTransaction(
      tx2,
      [initializerAccountKeypair, bettingMarketDataAccountKeypair],
      {
        skipPreflight: false,
        preflightCommitment: "singleGossip",
      }
    );
    console.log(bs58.encode(tx2.signature!));
    bettingMarketDataAccountPubkey = bettingMarketDataAccountKeypair.publicKey;
  }

  console.log("initBettingMarketWithParams", {
    "intializer account": initializerAccountKeypair.publicKey.toBase58(),
    "betting market data account": bettingMarketDataAccountPubkey.toBase58(),
    "usd token mint": usdTokenMintPubkey.toBase58(),
    "yes token mint": yesTokenMintAccountKeypair.publicKey.toBase58(),
    "no token mint": noTokenMintAccountKeypair.publicKey.toBase58(),
    "usd token account": usdTokenAccountKeypair.publicKey.toBase58(),
    "judge account": judgeAccountPubkey.toBase58(),
  });

  const initBettingMarketIx = new TransactionInstruction({
    programId: bettingMarketProgramId,
    keys: [
      {
        pubkey: initializerAccountKeypair.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: PDA,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: bettingMarketDataAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      {
        pubkey: usdTokenMintPubkey,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: yesTokenMintAccountKeypair.publicKey,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: noTokenMintAccountKeypair.publicKey,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: usdTokenAccountKeypair.publicKey,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: judgeAccountPubkey,
        isSigner: false,
        isWritable: false,
      },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(Uint8Array.of(2)),
  });

  const tx = new Transaction().add(initBettingMarketIx);
  // .add(createBettingMarketDataAccountIx);
  await connection.sendTransaction(
    tx,
    [
      initializerAccountKeypair,
      // bettingMarketDataAccountKeypair,
      yesTokenMintAccountKeypair,
      noTokenMintAccountKeypair,
      usdTokenAccountKeypair,
    ],
    {
      skipPreflight: false,
      preflightCommitment: "singleGossip",
    }
  );

  console.log("finished initBettingMarket transaction");
  console.log("transaction", bs58.encode(tx.signature!));
  console.log(
    "betting market data account",
    // bettingMarketDataAccountKeypair.publicKey.toBase58()
    bettingMarketDataAccountPubkey.toBase58()
  );

  return [tx, bettingMarketDataAccountKeypair];
};

export const getBettingMarketState = async (
  bettingMarketDataAccountPubkey: PublicKey
): Promise<BettingMarketState> => {
  const encodedBettingMarketState = (await connection.getAccountInfo(
    bettingMarketDataAccountPubkey,
    "singleGossip"
  ))!.data;
  const decodedBettingMarketState = BETTING_MARKET_DATA_LAYOUT.decode(
    encodedBettingMarketState
  ) as RawBettingMarketData;
  console.log(decodedBettingMarketState);
  const range = (start: number, stop: number, step = 1) =>
    Array(stop - start)
      .fill(start)
      .map((x, y) => x + y * step);
  const bettingMarketState: BettingMarketState = {
    isInitialized: !!decodedBettingMarketState.isInitialized,
    result: decodedBettingMarketState.result,
    yesTokenMint: new PublicKey(decodedBettingMarketState.yesTokenMint),
    noTokenMint: new PublicKey(decodedBettingMarketState.noTokenMint),
    usdTokenAccount: new PublicKey(decodedBettingMarketState.usdTokenAccount),
    strikePrice: new BN(
      decodedBettingMarketState.strikePrice,
      10,
      "le"
    ).toNumber(),
    judge: new PublicKey(decodedBettingMarketState.judge),
    buyAmountsForYesPrice: range(0, 101, 8).map((i) =>
      new BN(
        decodedBettingMarketState.buyAmountsForYesPrice.slice(i, i + 8),
        10,
        "le"
      ).toNumber()
    ),
    buyAmountsForNoPrice: range(0, 101, 8).map((i) =>
      new BN(
        decodedBettingMarketState.buyAmountsForNoPrice.slice(i, i + 8),
        10,
        "le"
      ).toNumber()
    ),
    payoutUserAccounts: range(0, 101, 32).map(
      (i) =>
        new PublicKey(
          decodedBettingMarketState.payoutUserAccounts.slice(i, i + 32)
        )
    ),
    payoutMints: range(0, 101, 32).map(
      (i) =>
        new PublicKey(decodedBettingMarketState.payoutMints.slice(i, i + 32))
    ),
    payoutAmounts: range(0, 101, 8).map((i) =>
      new BN(
        decodedBettingMarketState.payoutAmounts.slice(i, i + 8),
        10,
        "le"
      ).toNumber()
    ),
  };

  console.log("bettingMarketState", {
    isInitialized: bettingMarketState.isInitialized,
    result: bettingMarketState.result,
    yesTokenMint: bettingMarketState.yesTokenMint.toBase58(),
    noTokenMint: bettingMarketState.noTokenMint.toBase58(),
    usdTokenAccount: bettingMarketState.usdTokenAccount.toBase58(),
    strikePrice: bettingMarketState.strikePrice,
    judge: bettingMarketState.judge.toBase58(),
    buyAmountsForYesPrice: bettingMarketState.buyAmountsForYesPrice,
    buyAmountsForNoPrice: bettingMarketState.buyAmountsForNoPrice,
    payoutUserAccounts: bettingMarketState.payoutUserAccounts.map((pk) =>
      pk.toBase58()
    ),
    payoutMints: bettingMarketState.payoutMints.map((pk) => pk.toBase58()),
    payoutAmounts: bettingMarketState.payoutAmounts,
  });

  return bettingMarketState;
};

export const offerTrade = async (
  userAccountKeypair: Keypair,
  bettingMarketDataAccountPubkey: PublicKey,
  bettingMarketState: BettingMarketState,
  userUsdTokenAccountPubkey: PublicKey,
  userYesTokenAccountPubkey: PublicKey,
  userNoTokenAccountPubkey: PublicKey,
  is_yes: boolean,
  price: number,
  amount: number
) => {
  const usdTokenMintPubkey = USD_TOKEN_MINT;
  const yesTokenMintPubkey = bettingMarketState.yesTokenMint;
  const noTokenMintPubkey = bettingMarketState.noTokenMint;
  const bettingMarketUsdTokenAccountPubkey = bettingMarketState.usdTokenAccount;

  console.log("offerTrade", {
    "intializer account": userAccountKeypair.publicKey.toBase58(),
    "betting market data account": bettingMarketDataAccountPubkey.toBase58(),
    "usd token mint": usdTokenMintPubkey.toBase58(),
    "yes token mint": yesTokenMintPubkey.toBase58(),
    "no token mint": noTokenMintPubkey.toBase58(),
    "user usd token account": userUsdTokenAccountPubkey.toBase58(),
    "user yes token account": userYesTokenAccountPubkey.toBase58(),
    "user no token account": userNoTokenAccountPubkey.toBase58(),
    "betting market usd token account":
      bettingMarketUsdTokenAccountPubkey.toBase58(),
  });

  const offerTradeIx = new TransactionInstruction({
    programId: BETTING_MARKET_PROGRAM_ID,
    keys: [
      {
        pubkey: userAccountKeypair.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: PDA,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: bettingMarketDataAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: usdTokenMintPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: yesTokenMintPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: noTokenMintPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: userUsdTokenAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: userYesTokenAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: userNoTokenAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: bettingMarketUsdTokenAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(
      Uint8Array.of(
        3,
        is_yes ? 1 : 0,
        ...new BN(price).toArray("le", 8),
        ...new BN(amount).toArray("le", 8)
      )
    ),
  });

  const tx = new Transaction().add(offerTradeIx);
  await connection.sendTransaction(tx, [userAccountKeypair], {
    skipPreflight: false,
    preflightCommitment: "singleGossip",
  });

  console.log("finished offerTrade transaction");
  console.log("transaction", bs58.encode(tx.signature!));

  return tx;
};

export const payout = async (
  userAccountKeypair: Keypair,
  bettingMarketDataAccountPubkey: PublicKey,
  bettingMarketState: BettingMarketState,
  userUsdTokenAccountPubkey: PublicKey,
  userYesTokenAccountPubkey: PublicKey,
  userNoTokenAccountPubkey: PublicKey
) => {
  const usdTokenMintPubkey = USD_TOKEN_MINT;
  const yesTokenMintPubkey = bettingMarketState.yesTokenMint;
  const noTokenMintPubkey = bettingMarketState.noTokenMint;
  const bettingMarketUsdTokenAccountPubkey = bettingMarketState.usdTokenAccount;

  console.log("payout", {
    "user account": userAccountKeypair.publicKey.toBase58(),
    "betting market data account": bettingMarketDataAccountPubkey.toBase58(),
    "usd token mint": usdTokenMintPubkey.toBase58(),
    "yes token mint": yesTokenMintPubkey.toBase58(),
    "no token mint": noTokenMintPubkey.toBase58(),
    "user usd token account": userUsdTokenAccountPubkey.toBase58(),
    "user yes token account": userYesTokenAccountPubkey.toBase58(),
    "user no token account": userNoTokenAccountPubkey.toBase58(),
    "betting market usd token account":
      bettingMarketUsdTokenAccountPubkey.toBase58(),
  });

  const payoutIx = new TransactionInstruction({
    programId: BETTING_MARKET_PROGRAM_ID,
    keys: [
      {
        pubkey: userAccountKeypair.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: PDA,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: bettingMarketDataAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: usdTokenMintPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: yesTokenMintPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: noTokenMintPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: userUsdTokenAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: userYesTokenAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: userNoTokenAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: bettingMarketUsdTokenAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(Uint8Array.of(4)),
  });

  const tx = new Transaction().add(payoutIx);
  await connection.sendTransaction(tx, [userAccountKeypair], {
    skipPreflight: false,
    preflightCommitment: "singleGossip",
  });

  console.log("finished payout transaction");
  console.log("transaction", bs58.encode(tx.signature!));

  return tx;
};

export const freeMint = async (
  userAccountKeypair: Keypair,
  desiredTokenMintPubkey: PublicKey,
  userDesiredTokenAccountPubkey: PublicKey,
  amount: number
) => {
  console.log("freeMint", {
    "user account": userAccountKeypair.publicKey.toBase58(),
    "desired token mint": desiredTokenMintPubkey.toBase58(),
    "user desired token account": userDesiredTokenAccountPubkey.toBase58(),
  });

  const freeMintIx = new TransactionInstruction({
    programId: BETTING_MARKET_PROGRAM_ID,
    keys: [
      {
        pubkey: PDA,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: desiredTokenMintPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: userDesiredTokenAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(Uint8Array.of(5, ...new BN(amount).toArray("le", 8))),
  });

  const tx = new Transaction().add(freeMintIx);
  await connection.sendTransaction(tx, [userAccountKeypair], {
    skipPreflight: false,
    preflightCommitment: "singleGossip",
  });

  console.log("finished freeMint transaction");
  console.log("transaction", bs58.encode(tx.signature!));

  return tx;
};

export const judgeBettingMarketManually = async (
  userAccountKeypair: Keypair,
  bettingMarketDataAccount: PublicKey,
  result: number
) => {
  console.log("judgeBettingMarketManually", {
    result: result,
  });

  const judgeBettingMarketManuallyIx = new TransactionInstruction({
    programId: BETTING_MARKET_PROGRAM_ID,
    keys: [
      {
        pubkey: bettingMarketDataAccount,
        isSigner: false,
        isWritable: true,
      },
    ],
    data: Buffer.from(Uint8Array.of(6, ...new BN(result).toArray("le", 8))),
  });

  const tx = new Transaction().add(judgeBettingMarketManuallyIx);
  await connection.sendTransaction(tx, [userAccountKeypair], {
    skipPreflight: false,
    preflightCommitment: "singleGossip",
  });

  console.log("finished judgeBettingMarketManually transaction");
  console.log("transaction", bs58.encode(tx.signature!));

  return tx;
};

export const judgeBettingMarketOracle = async (
  userAccountKeypair: Keypair,
  bettingMarketDataAccount: PublicKey,
  priceAccountPubkey: PublicKey
) => {
  console.log("judgeBettingMarketOracle", {
    "price account": priceAccountPubkey.toBase58(),
  });

  const judgeBettingMarketOracleIx = new TransactionInstruction({
    programId: BETTING_MARKET_PROGRAM_ID,
    keys: [
      {
        pubkey: bettingMarketDataAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: priceAccountPubkey,
        isSigner: false,
        isWritable: false,
      },
    ],
    data: Buffer.from(Uint8Array.of(7)),
  });

  const tx = new Transaction().add(judgeBettingMarketOracleIx);
  await connection.sendTransaction(tx, [userAccountKeypair], {
    skipPreflight: false,
    preflightCommitment: "singleGossip",
  });

  console.log("finished judgeBettingMarketOracle transaction");
  console.log("transaction", bs58.encode(tx.signature!));

  return tx;
};

export const setStrikePrice = async (
  userAccountKeypair: Keypair,
  bettingMarketDataAccount: PublicKey,
  strikePrice: number
) => {
  console.log("setStrikePrice", {
    "strike price": strikePrice,
  });

  const setStrikePriceIx = new TransactionInstruction({
    programId: BETTING_MARKET_PROGRAM_ID,
    keys: [
      {
        pubkey: bettingMarketDataAccount,
        isSigner: false,
        isWritable: true,
      },
    ],
    data: Buffer.from(
      Uint8Array.of(8, ...new BN(strikePrice).toArray("le", 8))
    ),
  });

  const tx = new Transaction().add(setStrikePriceIx);
  await connection.sendTransaction(tx, [userAccountKeypair], {
    skipPreflight: false,
    preflightCommitment: "singleGossip",
  });

  console.log("finished setStrikePrice transaction");
  console.log("transaction", bs58.encode(tx.signature!));

  return tx;
};

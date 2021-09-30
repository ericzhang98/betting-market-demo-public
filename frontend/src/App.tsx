import React, { useState, useEffect } from "react";
import "./App.css";
import {
  Container,
  Row,
  Button,
  FormControl,
  Table,
  InputGroup,
} from "react-bootstrap";
import "bootstrap/dist/css/bootstrap.min.css";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  getBettingMarketState,
  initBettingMarket,
  offerTrade,
  payout,
  freeMint,
  judgeBettingMarketManually,
  judgeBettingMarketOracle,
  getBtcPriceData,
  setStrikePrice,
} from "./bettingMarket";
import {
  BETTING_MARKET_PROGRAM_ID,
  USD_TOKEN_MINT,
  keypairFromNums,
  findAssociatedTokenAddress,
  getTokenAccountBalanceString,
  BETTING_MARKET_DATA_ACCOUNT,
  NULL_PUBLIC_KEY,
  createAssociatedTokenAccount,
  connection,
  BTC_PRICE_ACCOUNT,
  ALICE_KEYPAIR,
  BOB_KEYPAIR,
  DEVMODE,
} from "./common";
import { BettingMarketState, formatBettingMarketInfo } from "./layout";
import OrderBook from "./OrderBook";

declare const window: any;

function App() {
  const [bettingMarketDataAccount, setBettingMarketDataAccount] =
    useState<PublicKey>();
  const [bettingMarketState, setBettingMarketState] =
    useState<BettingMarketState>();
  const [userKeypair, setUserKeypair] = useState<Keypair>();
  const [userUsdTokenAccount, setUserUsdTokenAccount] = useState<PublicKey>();
  const [userYesTokenAccount, setUserYesTokenAccount] = useState<PublicKey>();
  const [userNoTokenAccount, setUserNoTokenAccount] = useState<PublicKey>();
  const [userUsdTokenBalance, setUserUsdTokenBalance] = useState<string>();
  const [userYesTokenBalance, setUserYesTokenBalance] = useState<string>();
  const [userNoTokenBalance, setUserNoTokenBalance] = useState<string>();
  const [userAmount, setUserAmount] = useState<number>();
  const [userPrice, setUserPrice] = useState<number>();
  const [btcPriceData, setBtcPriceData] = useState<string>();

  const refreshBettingMarketState = async () => {
    if (!bettingMarketDataAccount) {
      return;
    }
    const bettingMarketState = await getBettingMarketState(
      bettingMarketDataAccount
    );
    setBettingMarketState(bettingMarketState);
  };

  const refreshTokenAccounts = async () => {
    if (!userKeypair) {
      return;
    }
    const userPubkey = userKeypair.publicKey;
    const yesTokenMint = bettingMarketState?.yesTokenMint;
    const noTokenMint = bettingMarketState?.noTokenMint;
    const associatedUsdTokenAddress = await findAssociatedTokenAddress(
      userPubkey,
      USD_TOKEN_MINT
    );
    setUserUsdTokenAccount(associatedUsdTokenAddress);
    setUserUsdTokenBalance(
      await getTokenAccountBalanceString(associatedUsdTokenAddress)
    );
    if (yesTokenMint && noTokenMint) {
      const associatedYesTokenAddress = await findAssociatedTokenAddress(
        userPubkey,
        yesTokenMint
      );
      setUserYesTokenAccount(associatedYesTokenAddress);
      setUserYesTokenBalance(
        await getTokenAccountBalanceString(associatedYesTokenAddress)
      );
      const associatedNoTokenAddress = await findAssociatedTokenAddress(
        userPubkey,
        noTokenMint
      );
      setUserNoTokenAccount(associatedNoTokenAddress);
      setUserNoTokenBalance(
        await getTokenAccountBalanceString(associatedNoTokenAddress)
      );
    }
  };

  const refreshBtcPriceDataAndSlot = async () => {
    let [btcPrice, btcConfidence, btcValidSlot] = await getBtcPriceData();
    setBtcPriceData(`$${Number(btcPrice).toFixed(2)}`);
    // setBtcPriceData(
    //   `$${Number(btcPrice).toFixed(2)} \xB1$${Number(btcConfidence).toFixed(
    //     2
    //   )} at slot ${btcValidSlot}`
    // );
  };

  const setupAliceClick = () => {
    setUserKeypair(ALICE_KEYPAIR);
    setBettingMarketDataAccount(BETTING_MARKET_DATA_ACCOUNT);
  };

  const setupBobClick = () => {
    setUserKeypair(BOB_KEYPAIR);
    setBettingMarketDataAccount(BETTING_MARKET_DATA_ACCOUNT);
  };

  const refreshClick = () => {
    refreshBettingMarketState();
    refreshTokenAccounts();
    refreshBtcPriceDataAndSlot();
  };

  const initBettingMarketClick = () => {
    console.log("initializing betting market");
    initBettingMarket(userKeypair!, BETTING_MARKET_PROGRAM_ID);
  };

  const createAssociatedTokenAccounts = async () => {
    console.log(
      "creating associated token accounts for yes token and no token"
    );
    try {
      await createAssociatedTokenAccount(
        userKeypair!,
        bettingMarketState!.yesTokenMint
      );
    } catch (err) {
      console.log("yes token account already exists");
    }
    try {
      await createAssociatedTokenAccount(
        userKeypair!,
        bettingMarketState!.noTokenMint
      );
    } catch (err) {
      console.log("no token account already exists");
    }
  };

  const buyYesTradeClick = () => {
    console.log("buy yes");
    offerToBuy(true, userPrice!);
  };

  const buyNoTradeClick = () => {
    console.log("buy no");
    offerToBuy(false, userPrice!);
  };

  const sellYesTradeClick = () => {
    console.log("sell yes");
    offerToBuy(false, 100 - userPrice!);
  };

  const sellNoTradeClick = () => {
    console.log("sell no");
    offerToBuy(true, 100 - userPrice!);
  };

  const offerToBuy = (is_yes: boolean, price: number) => {
    if (price <= 0 || price >= 100) {
      console.log("bad price", price);
      return;
    }
    offerTrade(
      userKeypair!,
      bettingMarketDataAccount!,
      bettingMarketState!,
      userUsdTokenAccount!,
      userYesTokenAccount!,
      userNoTokenAccount!,
      is_yes,
      price,
      userAmount!
    );
  };

  const payoutClick = () => {
    console.log("payout");
    payout(
      userKeypair!,
      bettingMarketDataAccount!,
      bettingMarketState!,
      userUsdTokenAccount!,
      userYesTokenAccount!,
      userNoTokenAccount!
    );
  };

  const freeYesMintClick = () => {
    console.log("free yes mint");
    freeMint(
      userKeypair!,
      bettingMarketState!.yesTokenMint,
      userYesTokenAccount!,
      userAmount!
    );
  };

  const freeNoMintClick = () => {
    console.log("free no mint");
    freeMint(
      userKeypair!,
      bettingMarketState!.noTokenMint,
      userNoTokenAccount!,
      userAmount!
    );
  };

  const judgeBettingMarketManuallyClick = () => {
    console.log("judge betting market manually");
    judgeBettingMarketManually(
      userKeypair!,
      bettingMarketDataAccount!,
      userAmount!
    );
  };

  const judgeBettingMarketOracleClick = () => {
    console.log("judge betting market oracle");
    judgeBettingMarketOracle(
      userKeypair!,
      bettingMarketDataAccount!,
      BTC_PRICE_ACCOUNT
    );
  };

  const setStrikePriceClick = () => {
    console.log("set strike price");
    setStrikePrice(userKeypair!, bettingMarketDataAccount!, userAmount!);
  };

  useEffect(() => {
    console.log("use effect");
    // connection.onSlotChange((slotInfo) => {
    //   console.log(slotInfo.slot);
    //   refreshBtcPriceDataAndSlot();
    // });
  }, []);
  useEffect(() => {
    refreshBettingMarketState();
  }, [bettingMarketDataAccount]);
  useEffect(() => {
    refreshTokenAccounts();
  }, [userKeypair, bettingMarketState]);

  // useEffect(() => {
  //   const isPhantomInstalled = window.solana && window.solana.isPhantom;
  //   if (isPhantomInstalled) {
  //     console.log("phantom installed", window.solana);
  //     window.solana.connect();
  //     window.solana.on("connect", () => console.log("connected!"));
  //   } else {
  //     console.log("phantom not installed", window.solana);
  //   }
  // }, []);

  return (
    <div className="App">
      <Container>
        <Row>
          <h3 style={{ marginTop: 20 }}>
            Betting market address: {bettingMarketDataAccount?.toBase58()}
          </h3>
          <Table striped bordered hover>
            <thead>
              <tr>
                <th style={{ width: 100 }}>BTC price from Pyth</th>
                <th style={{ width: 100 }}>Strike price</th>
                <th style={{ width: 100 }}>Result</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>{btcPriceData}</td>
                <td>
                  {bettingMarketState && "$" + bettingMarketState?.strikePrice}
                </td>
                <td>
                  {bettingMarketState &&
                    { 0: "undecided", 1: "YES", 2: "NO" }[
                      bettingMarketState.result
                    ]}
                </td>
              </tr>
            </tbody>
          </Table>
          <OrderBook
            sellEntries={
              bettingMarketState?.buyAmountsForNoPrice
                .map((amount, price) => [price, amount])
                .filter((arr) => arr[1] !== 0)
                .map((arr) => {
                  return { price: (100 - arr[0]) / 100, size: arr[1] };
                }) || []
            }
            buyEntries={
              bettingMarketState?.buyAmountsForYesPrice
                .map((amount, price) => [price, amount])
                .filter((arr) => arr[1] !== 0)
                .map((arr) => {
                  return { price: arr[0] / 100, size: arr[1] };
                }) || []
            }
          />
          <h3 style={{ marginTop: 20 }}>Payouts</h3>
          <Table striped bordered hover>
            <thead>
              <tr>
                <th style={{ width: 100 }}>User account</th>
                <th style={{ width: 100 }}>Token mint</th>
                <th style={{ width: 100 }}>Amount</th>
              </tr>
            </thead>
            <tbody>
              {bettingMarketState &&
                bettingMarketState.payoutUserAccounts
                  .map((pk, i) => {
                    return { pk, i };
                  })
                  .filter(
                    (info) => info.pk.toBase58() !== NULL_PUBLIC_KEY.toBase58()
                  )
                  .map((info, _) => {
                    const i = info.i;
                    let payoutUserAccount =
                      bettingMarketState.payoutUserAccounts[i].toBase58();
                    let payoutMint =
                      bettingMarketState.payoutMints[i].toBase58();
                    let nameMapping: { [key: string]: string } = {};
                    nameMapping[bettingMarketState.yesTokenMint.toBase58()] =
                      "YES";
                    nameMapping[bettingMarketState.noTokenMint.toBase58()] =
                      "NO";
                    nameMapping[USD_TOKEN_MINT.toBase58()] = "USD";
                    let payoutAmount = bettingMarketState.payoutAmounts[i];
                    return (
                      <tr>
                        <td>{payoutUserAccount}</td>
                        <td>{nameMapping[payoutMint]}</td>
                        <td>{payoutAmount}</td>
                      </tr>
                    );
                  })}
            </tbody>
          </Table>
          <h3 style={{ marginTop: 20 }}>
            Current user account: {userKeypair?.publicKey.toBase58()}
          </h3>
          <Table striped bordered hover>
            <thead>
              <tr>
                <th style={{ width: 100 }}>USD</th>
                <th style={{ width: 100 }}>YES</th>
                <th style={{ width: 100 }}>NO</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>{userUsdTokenBalance}</td>
                <td>{userYesTokenBalance}</td>
                <td>{userNoTokenBalance}</td>
              </tr>
            </tbody>
          </Table>
          <InputGroup>
            <InputGroup.Text>Betting market address</InputGroup.Text>
            <FormControl
              placeholder={bettingMarketDataAccount?.toBase58() || ""}
              onChange={(e) => {
                try {
                  setBettingMarketDataAccount(new PublicKey(e.target.value));
                } catch (err) {}
              }}
            />
          </InputGroup>
          <InputGroup>
            <InputGroup.Text>Amount</InputGroup.Text>
            <FormControl
              onChange={(e) => setUserAmount(Number(e.target.value))}
            />
          </InputGroup>
          <InputGroup>
            <InputGroup.Text>Price in minor units</InputGroup.Text>
            <FormControl
              onChange={(e) => setUserPrice(Number(e.target.value))}
            />
          </InputGroup>
          <Button onClick={refreshClick}>Refresh</Button>{" "}
          <Button onClick={buyYesTradeClick}>Buy YES</Button>{" "}
          <Button onClick={sellYesTradeClick}>Sell YES</Button>{" "}
          <Button onClick={buyNoTradeClick}>Buy NO</Button>{" "}
          <Button onClick={sellNoTradeClick}>Sell NO</Button>{" "}
          <Button onClick={payoutClick}>Payout</Button>{" "}
          <Button onClick={judgeBettingMarketOracleClick}>
            Judge Betting Market with Pyth Price Oracle
          </Button>
          <Button onClick={setupAliceClick}>Setup Alice</Button>{" "}
          <Button onClick={setupBobClick}>Setup Bob</Button>{" "}
          <Button onClick={initBettingMarketClick}>Init Betting Market</Button>{" "}
          <Button onClick={setStrikePriceClick}>Set Strike Price</Button>
          <Button onClick={judgeBettingMarketManuallyClick}>
            Judge Betting Market Manually
          </Button>
          <div hidden={!DEVMODE}>
            <FormControl
              placeholder={
                userKeypair?.secretKey.toString() || "User private key"
              }
              onChange={(e) => {
                try {
                  setUserKeypair(keypairFromNums(e.target.value));
                  refreshTokenAccounts();
                } catch (err) {
                  console.log("bad user private key");
                }
              }}
            />
            <Button onClick={createAssociatedTokenAccounts}>
              Create Associated Token Accounts
            </Button>{" "}
            <Button onClick={freeYesMintClick}>Free YES Mint</Button>{" "}
            <Button onClick={freeNoMintClick}>Free NO Mint</Button>{" "}
            <p>BTC price data: {btcPriceData}</p>
            <p>
              Betting market data account:{" "}
              {bettingMarketDataAccount?.toBase58()}
            </p>
            <p style={{ overflowWrap: "break-word" }}>
              Betting market state:{" "}
              {bettingMarketState &&
                JSON.stringify(formatBettingMarketInfo(bettingMarketState))}
            </p>
            <p>User public key: {userKeypair?.publicKey.toBase58()}</p>
            <p>User USD token account: {userUsdTokenAccount?.toBase58()}</p>
            <p>User USD token balance: {userUsdTokenBalance}</p>
            <p>User YES token account: {userYesTokenAccount?.toBase58()}</p>
            <p>User YES token balance: {userYesTokenBalance}</p>
            <p>User NO token account: {userNoTokenAccount?.toBase58()}</p>
            <p>User NO token balance: {userNoTokenBalance}</p>
            <p>
              YES orders:{" "}
              {bettingMarketState?.buyAmountsForYesPrice
                .map((amount, price) => [price, amount])
                .filter((arr) => arr[1] !== 0)
                .map((arr) => arr[0] + ": " + arr[1] + "; ")}
            </p>
            <p>
              NO orders:{" "}
              {bettingMarketState?.buyAmountsForNoPrice
                .map((amount, price) => [price, amount])
                .filter((arr) => arr[1] !== 0)
                .map((arr) => arr[0] + ": " + arr[1] + "; ")}
            </p>
            <p>Payouts:</p>
            {bettingMarketState &&
              bettingMarketState.payoutUserAccounts
                .map((pk, i) => {
                  return { pk, i };
                })
                .filter(
                  (info) => info.pk.toBase58() !== NULL_PUBLIC_KEY.toBase58()
                )
                .map((info, _) => {
                  const i = info.i;
                  let payoutUserAccount =
                    bettingMarketState.payoutUserAccounts[i].toBase58();
                  let payoutMint = bettingMarketState.payoutMints[i].toBase58();
                  let nameMapping: { [key: string]: string } = {};
                  nameMapping[bettingMarketState.yesTokenMint.toBase58()] =
                    "YES";
                  nameMapping[bettingMarketState.noTokenMint.toBase58()] = "NO";
                  nameMapping[USD_TOKEN_MINT.toBase58()] = "USD";
                  let payoutAmount = bettingMarketState.payoutAmounts[i];
                  return (
                    <p key={i}>
                      {"(" +
                        payoutUserAccount +
                        ", " +
                        nameMapping[payoutMint] +
                        ", " +
                        payoutAmount +
                        ")"}
                    </p>
                  );
                })}
          </div>
        </Row>
      </Container>
    </div>
  );
}

export default App;

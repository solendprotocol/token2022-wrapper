import * as web3 from "@solana/web3.js";
import * as token2022WrapperSdk from "../src";
import { createKeypair, delay } from "./utils/helpers";
import { initializeMints2022 } from "./utils/token2022utils";
import { createAssociatedTokenAccountIdempotent } from "@solana/spl-token";
import BN from "bn.js";

const main = async () => {
  // Create a token and mint it to users

  const connection = new web3.Connection(
    `https://api.devnet.solana.com`,
    "confirmed"
  );

  let userA = await createKeypair(connection, true);
  let userB = await createKeypair(connection, true);

  let token2022TokenInfo = await initializeMints2022(
    connection,
    1,
    [9],
    [100],
    [userA.publicKey, userB.publicKey]
  );

  let token2022Mint = token2022TokenInfo.tokens[0];
  let token2022Decimals = token2022TokenInfo.decimals[0];
  let token2022Authority = token2022TokenInfo.authority;

  await delay(1_000);

  // Initialize a wrapper token mint
  let initializeIx =
    token2022WrapperSdk.createInitializeWrapperTokenInstruction(
      userB.publicKey,
      token2022Mint
    );

  let initializeTx = new web3.Transaction().add(initializeIx);
  initializeTx.recentBlockhash = (
    await connection.getLatestBlockhash()
  ).blockhash;
  let initializeSig = await web3.sendAndConfirmTransaction(
    connection,
    initializeTx,
    [userB]
  );
  console.log(
    `Wrapper tokens initialized: https://solscan.io/tx/${initializeSig}?cluster=devnet`
  );

  await delay(2_000);

  // Get wrapper token mint
  let wrapperTokenMint = token2022WrapperSdk.getWrapperTokenMint(token2022Mint);

  const userBToken2022Account = await createAssociatedTokenAccountIdempotent(
    connection,
    userB,
    token2022Mint,
    userB.publicKey,
    {},
    token2022WrapperSdk.TOKEN_2022_PROGRAM_ID
  );

  // Should be X
  let userT22BalanceBeforeDeposit = (
    await connection.getTokenAccountBalance(userBToken2022Account)
  ).value.uiAmount;

  console.log("User T22 balance before deposit: ", userT22BalanceBeforeDeposit);

  let depositAmount = new BN(100).mul(new BN(10 ** token2022Decimals));

  let depositIx =
    await token2022WrapperSdk.createDepositAndMintWrapperTokensInstruction(
      userB.publicKey,
      userBToken2022Account,
      token2022Mint,
      depositAmount
    );

  let depositTx = new web3.Transaction().add(depositIx);
  depositTx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  let depositSig = await web3.sendAndConfirmTransaction(connection, depositTx, [
    userB,
  ]);
  console.log(
    `Wrapper tokens minted: https://solscan.io/tx/${depositSig}?cluster=devnet`
  );

  const userBWrapperTokenAccount = await createAssociatedTokenAccountIdempotent(
    connection,
    userB,
    wrapperTokenMint,
    userB.publicKey,
    {},
    token2022WrapperSdk.TOKEN_PROGRAM_ID
  );

  await delay(2_000);

  // Should be X - 100
  let userT22BalanceAfterDeposit = (
    await connection.getTokenAccountBalance(userBToken2022Account)
  ).value.uiAmount;
  // Should be 99
  let userWrapperBalanceAfterDeposit = (
    await connection.getTokenAccountBalance(userBWrapperTokenAccount)
  ).value.uiAmount;

  console.log("User T22 balance after deposit: ", userT22BalanceAfterDeposit);
  console.log(
    "User wrapper balance after deposit: ",
    userWrapperBalanceAfterDeposit
  );

  let withdrawAmount = new BN(99).mul(new BN(10 ** token2022Decimals));

  let withdrawIx =
    await token2022WrapperSdk.createWithdrawAndBurnWrapperTokensInstruction(
      userB.publicKey,
      userBToken2022Account,
      token2022Mint,
      withdrawAmount
    );

  let withdrawTx = new web3.Transaction().add(withdrawIx);
  withdrawTx.recentBlockhash = (
    await connection.getLatestBlockhash()
  ).blockhash;
  let withdrawSig = await web3.sendAndConfirmTransaction(
    connection,
    withdrawTx,
    [userB]
  );
  console.log(
    `Wrapper tokens burned: https://solscan.io/tx/${withdrawSig}?cluster=devnet`
  );

  // Should be X - 1.99
  let userT22BalanceAfterWithdraw = (
    await connection.getTokenAccountBalance(userBToken2022Account)
  ).value.uiAmount;
  // Should be 0
  let userWrapperBalanceAfterWithdraw = (
    await connection.getTokenAccountBalance(userBWrapperTokenAccount)
  ).value.uiAmount;

  console.log("User T22 balance after withdraw: ", userT22BalanceAfterWithdraw);
  console.log(
    "User wrapper balance after withdraw: ",
    userWrapperBalanceAfterWithdraw
  );
};

main();

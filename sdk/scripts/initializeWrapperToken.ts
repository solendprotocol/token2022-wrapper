import * as web3 from "@solana/web3.js";
import * as token2022WrapperSdk from "../src";
import BN from "bn.js";
import { delay } from "../tests/utils";

require("dotenv").config();

const main = async () => {
  const connection = new web3.Connection(
    `https://api.mainnet-beta.solana.com`,
    "confirmed"
  );

  // TODO - Place the Token22 mint for which you wanna initialize the wrapper mint here
  let token2022Mint = new web3.PublicKey(
    "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo"
  );

  //@ts-ignore
  let privateKeyArray = JSON.parse(process.env.PRIVATE_KEY);

  let payerKeypair = web3.Keypair.fromSecretKey(
    Uint8Array.from(privateKeyArray)
  );

  await delay(1_000);

  let ixs = token2022WrapperSdk.requestComputeUnits(500_000, 100_000);

  // Initialize a wrapper token mint
  let initializeIx =
    token2022WrapperSdk.createInitializeWrapperTokenInstruction(
      payerKeypair.publicKey,
      token2022Mint
    );

  ixs.push(initializeIx);

  const expectedWrapperTokenMint =
    token2022WrapperSdk.getWrapperTokenMint(token2022Mint);

  let initializeTx = new web3.Transaction();

  ixs.forEach((ix) => {
    initializeTx.add(ix);
  });

  console.log(
    "Initializing wrapper token mint for: ",
    token2022Mint.toString()
  );
  console.log("Initializer: ", payerKeypair.publicKey.toString());

  console.log("Token 2022 mint: ", token2022Mint.toString());
  console.log("Wrapper token mint: ", expectedWrapperTokenMint.toString());

  initializeTx.recentBlockhash = (
    await connection.getLatestBlockhash()
  ).blockhash;
  let initializeSig = await web3.sendAndConfirmTransaction(
    connection,
    initializeTx,
    [payerKeypair]
  );

  console.log(
    `Wrapper tokens initialized: https://solscan.io/tx/${initializeSig}`
  );

  await delay(2_000);
};

main();

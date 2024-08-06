import * as web3 from "@solana/web3.js";
import BN from "bn.js";
import { createKeypair } from "./helpers";
import {
  createAssociatedTokenAccountIdempotent,
  createInitializeMintInstruction,
  createInitializeTransferFeeConfigInstruction,
  ExtensionType,
  getMintLen,
  mintTo,
  mintToChecked,
} from "@solana/spl-token";
import { TOKEN_2022_PROGRAM_ID } from "../../src";

export const initializeMints2022 = async (
  connection: web3.Connection,
  numMints: number,
  decimals: number[],
  transferFeeBps: number[],
  recipients?: web3.PublicKey[]
) => {
  console.log(`Starting to create ${numMints} T22 mints`);
  let mints: web3.PublicKey[] = [];

  let authorityKeypair = await createKeypair(connection, true);
  const extensions = [ExtensionType.TransferFeeConfig];
  const mintLen = getMintLen(extensions);
  const mintLamports = await connection.getMinimumBalanceForRentExemption(
    mintLen
  );

  let i = 0;
  for (let _ of Array(numMints)) {
    // Step 2 - Create a New Token

    const mintKeypair = web3.Keypair.generate();
    const tokenMint = mintKeypair.publicKey;

    const decimal = decimals[i];

    const mintTransaction = new web3.Transaction().add(
      web3.SystemProgram.createAccount({
        fromPubkey: authorityKeypair.publicKey,
        newAccountPubkey: tokenMint,
        space: mintLen,
        lamports: mintLamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeTransferFeeConfigInstruction(
        tokenMint,
        authorityKeypair.publicKey,
        authorityKeypair.publicKey,
        transferFeeBps[i],
        BigInt(transferFeeBps[i] * 10 ** decimal),
        TOKEN_2022_PROGRAM_ID
      ),
      // createInitializeNonTransferableMintInstruction(mint, TOKEN_2022_PROGRAM_ID),
      createInitializeMintInstruction(
        tokenMint,
        decimal,
        authorityKeypair.publicKey,
        null,
        TOKEN_2022_PROGRAM_ID
      )
    );

    const newTokenTx = await web3.sendAndConfirmTransaction(
      connection,
      mintTransaction,
      [authorityKeypair, mintKeypair],
      undefined
    );

    console.log("Creating T22 token: ", tokenMint.toString());

    let amount = new BN(10_000).mul(new BN(10 ** decimals[i]));

    recipients?.forEach(async (recipient) => {
      const sourceAccount = await createAssociatedTokenAccountIdempotent(
        connection,
        authorityKeypair,
        tokenMint,
        recipient,
        {},
        TOKEN_2022_PROGRAM_ID
      );
      const mintSig = await mintToChecked(
        connection,
        authorityKeypair,
        tokenMint,
        sourceAccount,
        authorityKeypair,
        amount.toNumber(),
        decimal,
        undefined,
        undefined,
        TOKEN_2022_PROGRAM_ID
      );

      console.log(
        `Minted 10,000 tokens to: ${sourceAccount.toString()} owned by: ${recipient.toString()}`
      );
    });
    mints.push(tokenMint);
    i++;
  }
  return {
    tokens: mints,
    decimals,
    authority: authorityKeypair,
  };
};

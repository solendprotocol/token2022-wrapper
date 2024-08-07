import * as web3 from "@solana/web3.js";
import BN from "bn.js";
import { createKeypair } from "./helpers";
import {
  createAssociatedTokenAccount,
  createAssociatedTokenAccountIdempotent,
  createMint,
  getAssociatedTokenAddress,
  mintToChecked,
} from "@solana/spl-token";

export const initializeMints = async (
  connection: web3.Connection,
  numMints: number,
  decimals: number[],
  recipients?: web3.PublicKey[]
) => {
  console.log(`Starting to create ${numMints} SPL mints`);
  let mints: web3.PublicKey[] = [];

  let keypair = await createKeypair(connection, true);

  let i = 0;
  for (let _ of Array(numMints)) {
    const tokenMint = await createMint(
      connection,
      keypair,
      keypair.publicKey,
      keypair.publicKey,
      decimals[i]
    );
    console.log("Creating SPL token: ", tokenMint.toString());

    const amount = new BN(10_000).mul(new BN(10 ** decimals[i]));

    if (recipients && recipients.length > 0) {
      for (let recipient of recipients) {
        const ataAddr = await createAssociatedTokenAccountIdempotent(
          connection,
          keypair,
          tokenMint,
          recipient
        );

        await mintToChecked(
          connection,
          keypair,
          tokenMint,
          ataAddr,
          keypair,
          amount.toNumber(),
          decimals[i]
        );

        console.log("Minted 10,000 to: ", recipient.toString());
      }
    }

    mints.push(tokenMint);
    i++;
  }
  return {
    tokens: mints,
    decimals,
    authority: keypair,
  };
};

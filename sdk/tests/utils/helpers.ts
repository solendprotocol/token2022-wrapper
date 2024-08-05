import * as web3 from "@solana/web3.js";

require("dotenv").config();

export const createKeypair = async (
    connection: web3.Connection
) => {
    const keypair = new web3.Keypair();

    //@ts-ignore
    let privateKeyArray = JSON.parse(process.env.PRIVATE_KEY);

    let privateKeypair = web3.Keypair.fromSecretKey(
      Uint8Array.from(privateKeyArray)
    );

    const privateKeypairBalance = await connection.getBalance(privateKeypair.publicKey);

    const tx = new web3.Transaction().add(
      web3.SystemProgram.transfer({
        fromPubkey: privateKeypair.publicKey,
        toPubkey: keypair.publicKey,
        lamports: 0.5 * web3.LAMPORTS_PER_SOL
      })
    );
    let status = await web3.sendAndConfirmTransaction(
      connection,
      tx,
      [privateKeypair]
    );

    return keypair;
};
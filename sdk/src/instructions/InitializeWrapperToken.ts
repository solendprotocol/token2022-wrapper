import * as web3 from "@solana/web3.js";
import { TokenWrapperInstruction } from "../types";
import {
  PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  SYSTEM_PROGRAM_ID,
  RENT_SYSVAR,
} from "../constants";
import {
  getReserveAuthority,
  getReserveAuthorityTokenAccount,
  getWrapperTokenMint,
} from "../utils";

const BufferLayout = require("buffer-layout");

export const createInitializeWrapperTokenInstruction = (
  payer: web3.PublicKey,
  token2022Mint: web3.PublicKey
): web3.TransactionInstruction => {
  const dataLayout = BufferLayout.struct([BufferLayout.u8("instruction")]);

  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: TokenWrapperInstruction.InitializeWrapperToken,
    },
    data
  );

  const wrapperTokenMint = getWrapperTokenMint(token2022Mint);
  const reserveAuthority = getReserveAuthority(token2022Mint);
  const reserveAuthorityTokenAccount =
    getReserveAuthorityTokenAccount(token2022Mint);

  const keys = [
    {
      pubkey: payer,
      isSigner: true,
      isWritable: true,
    },
    {
      pubkey: token2022Mint,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: wrapperTokenMint,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: reserveAuthority,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: reserveAuthorityTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: TOKEN_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: TOKEN_2022_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: SYSTEM_PROGRAM_ID,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: RENT_SYSVAR,
      isSigner: false,
      isWritable: false,
    },
  ];

  return new web3.TransactionInstruction({
    keys,
    programId: PROGRAM_ID,
    data,
  });
};

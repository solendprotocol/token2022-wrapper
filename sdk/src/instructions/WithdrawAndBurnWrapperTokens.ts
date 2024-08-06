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
import * as Layout from "../types/layout";
import BN from "bn.js";
import { getAssociatedTokenAddress } from "@solana/spl-token";

const BufferLayout = require("buffer-layout");

export const createWithdrawAndBurnWrapperTokensInstruction = async (
  userAuthority: web3.PublicKey,
  userToken2022TokenAccount: web3.PublicKey,
  token2022Mint: web3.PublicKey,
  amount: number | BN
): Promise<web3.TransactionInstruction> => {
  const dataLayout = BufferLayout.struct([
    BufferLayout.u8("instruction"),
    Layout.uint64("amount"),
  ]);

  const data = Buffer.alloc(dataLayout.span);
  dataLayout.encode(
    {
      instruction: TokenWrapperInstruction.WithdrawAndBurnWrapperTokens,
      amount: new BN(amount),
    },
    data
  );

  const wrapperTokenMint = getWrapperTokenMint(token2022Mint);
  const reserveAuthority = getReserveAuthority(token2022Mint);
  const reserveAuthorityTokenAccount =
    getReserveAuthorityTokenAccount(token2022Mint);
  const userWrapperTokenAccount = await getAssociatedTokenAddress(
    wrapperTokenMint,
    userAuthority,
    false
  );

  const keys = [
    {
      pubkey: userAuthority,
      isSigner: true,
      isWritable: true,
    },
    {
      pubkey: reserveAuthority,
      isSigner: false,
      isWritable: false,
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
      pubkey: userWrapperTokenAccount,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: userToken2022TokenAccount,
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

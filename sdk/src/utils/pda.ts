import * as web3 from "@solana/web3.js";
import {
  PROGRAM_ID,
  RESERVE_AUTHORITY_SEED,
  RESERVE_AUTHORITY_TOKEN_ACCOUNT_SEED,
} from "../constants";
import { WRAPPER_TOKEN_MINT_SEED } from "../constants";

export const getWrapperTokenMint = (
  token2022Mint: web3.PublicKey
): web3.PublicKey => {
  const [wrapperTokenMint, _] = web3.PublicKey.findProgramAddressSync(
    [WRAPPER_TOKEN_MINT_SEED, token2022Mint.toBuffer()],
    PROGRAM_ID
  );

  return wrapperTokenMint;
};

export const getReserveAuthority = (
  token2022Mint: web3.PublicKey
): web3.PublicKey => {
  const [reserveAuthority, _] = web3.PublicKey.findProgramAddressSync(
    [RESERVE_AUTHORITY_SEED, token2022Mint.toBuffer()],
    PROGRAM_ID
  );

  return reserveAuthority;
};

export const getReserveAuthorityTokenAccount = (
  token2022Mint: web3.PublicKey
): web3.PublicKey => {
  let reserveAuthority = getReserveAuthority(token2022Mint);

  const [reserveAuthorityTokenAccount, _] =
    web3.PublicKey.findProgramAddressSync(
      [
        RESERVE_AUTHORITY_TOKEN_ACCOUNT_SEED,
        token2022Mint.toBuffer(),
        reserveAuthority.toBuffer(),
      ],
      PROGRAM_ID
    );

  return reserveAuthorityTokenAccount;
};

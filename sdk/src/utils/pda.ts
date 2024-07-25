import * as web3 from "@solana/web3.js";
import { FREEZE_AUTHORITY_SEED, MINT_AUTHORITY_SEED, PROGRAM_ID, RESERVE_AUTHORITY_SEED, RESERVE_AUTHORITY_TOKEN_ACCOUNT_SEED } from "../constants";
import { WRAPPER_TOKEN_MINT_SEED } from "../constants";

export const getWrapperTokenMint = async(
    token2022Mint: web3.PublicKey
): Promise<web3.PublicKey> => {
    const [wrapperTokenMint, _] = web3.PublicKey.findProgramAddressSync(
        [
            WRAPPER_TOKEN_MINT_SEED,
            token2022Mint.toBuffer()
        ],
        PROGRAM_ID
    );

    return wrapperTokenMint;
}

export const getReserveAuthority = async(
    token2022Mint: web3.PublicKey
): Promise<web3.PublicKey> => {
    const [reserveAuthority, _] = web3.PublicKey.findProgramAddressSync(
        [
            RESERVE_AUTHORITY_SEED,
            token2022Mint.toBuffer()
        ],
        PROGRAM_ID
    );

    return reserveAuthority;
}

export const getReserveAuthorityTokenAccount = async(
    token2022Mint: web3.PublicKey
): Promise<web3.PublicKey> => {
    let reserveAuthority = await getReserveAuthority(token2022Mint);

    const [reserveAuthorityTokenAccount, _] = web3.PublicKey.findProgramAddressSync(
        [
            RESERVE_AUTHORITY_TOKEN_ACCOUNT_SEED,
            token2022Mint.toBuffer(),
            reserveAuthority.toBuffer()
        ],
        PROGRAM_ID
    );

    return reserveAuthorityTokenAccount;
}

export const getMintAuthority = async(
    token2022Mint: web3.PublicKey
): Promise<web3.PublicKey> => {
    const [mintAuthority, _] = web3.PublicKey.findProgramAddressSync(
        [
            MINT_AUTHORITY_SEED,
            token2022Mint.toBuffer()
        ],
        PROGRAM_ID
    );

    return mintAuthority;
}

export const getFreezeAuthority = async(
    token2022Mint: web3.PublicKey
): Promise<web3.PublicKey> => {
    const [freezeAuthority, _] = web3.PublicKey.findProgramAddressSync(
        [
            FREEZE_AUTHORITY_SEED,
            token2022Mint.toBuffer()
        ],
        PROGRAM_ID
    );

    return freezeAuthority;
}
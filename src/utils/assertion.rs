use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, sysvar,
};

use crate::error::TokenWrapperError;

use super::{
    get_reserve_authority, get_reserve_authority_token_account, get_token_freeze_authority,
    get_token_mint_authority, get_vanilla_token_mint,
};

#[track_caller]
#[inline(always)]
pub fn assert_with_msg(v: bool, err: impl Into<ProgramError>, msg: &str) -> ProgramResult {
    if v {
        Ok(())
    } else {
        let caller = std::panic::Location::caller();
        msg!("{}. \n{}", msg, caller);
        Err(err.into())
    }
}

pub fn assert_token_program(p: Pubkey) -> ProgramResult {
    assert_with_msg(
        p == spl_token::ID,
        TokenWrapperError::UnexpectedTokenProgram,
        "Invalid Token program pubkey passed",
    )
}

pub fn assert_associated_token_program(p: Pubkey) -> ProgramResult {
    assert_with_msg(
        p == spl_associated_token_account::ID,
        TokenWrapperError::UnexpectedTokenProgram,
        "Invalid Token program pubkey passed",
    )
}

pub fn assert_token_2022_program(p: Pubkey) -> ProgramResult {
    assert_with_msg(
        p == spl_token_2022::ID,
        TokenWrapperError::UnexpectedTokenProgram,
        "Invalid Token 2022 program pubkey passed",
    )
}

pub fn assert_system_program(p: Pubkey) -> ProgramResult {
    assert_with_msg(
        p == solana_program::system_program::ID,
        TokenWrapperError::UnexpectedTokenProgram,
        "Invalid System program pubkey passed",
    )
}

pub fn assert_rent(p: Pubkey) -> ProgramResult {
    assert_with_msg(
        p == sysvar::rent::ID,
        TokenWrapperError::UnexpectedRent,
        "Invalid Rent pubkey passed",
    )
}

pub fn assert_vanilla_token_mint(
    token_2022_mint: Pubkey,
    program_id: Pubkey,
    actual_vanilla_token_mint: Pubkey,
) -> ProgramResult {
    let (expected_vanilla_token_mint, _, _) = get_vanilla_token_mint(token_2022_mint, program_id);

    assert_with_msg(
        expected_vanilla_token_mint == actual_vanilla_token_mint,
        TokenWrapperError::UnexpectedVanillaToken,
        "Invalid vanilla token mint passed",
    )
}

pub fn assert_reserve_authority(
    token_2022_mint: Pubkey,
    program_id: Pubkey,
    actual_reserve_authority: Pubkey,
) -> ProgramResult {
    let (expected_reserve_authority, _, _) = get_reserve_authority(token_2022_mint, program_id);

    assert_with_msg(
        expected_reserve_authority == actual_reserve_authority,
        TokenWrapperError::UnexpectedReserveAuthority,
        "Invalid reserve authority passed",
    )
}

pub fn assert_reserve_authority_token_account(
    token_2022_mint: Pubkey,
    owner: Pubkey,
    program_id: Pubkey,
    actual_reserve_authority_token_account: Pubkey,
) -> ProgramResult {
    let (expected_reserve_authority_token_account, _, _) =
        get_reserve_authority_token_account(token_2022_mint, owner, program_id);

    assert_with_msg(
        expected_reserve_authority_token_account == actual_reserve_authority_token_account,
        TokenWrapperError::UnexpectedReserveTokenAccount,
        "Invalid reserve authority token account passed",
    )
}

pub fn assert_mint_authority(
    token_mint: Pubkey,
    program_id: Pubkey,
    actual_mint_authority: Pubkey,
) -> ProgramResult {
    let (expected_mint_authority, _, _) = get_token_mint_authority(token_mint, program_id);

    assert_with_msg(
        expected_mint_authority == actual_mint_authority,
        TokenWrapperError::UnexpectedMintAuthority,
        "Invalid mint authority passed",
    )
}

pub fn assert_freeze_authority(
    token_mint: Pubkey,
    program_id: Pubkey,
    actual_freeze_authority: Pubkey,
) -> ProgramResult {
    let (expected_freeze_authority, _, _) = get_token_freeze_authority(token_mint, program_id);

    assert_with_msg(
        expected_freeze_authority == actual_freeze_authority,
        TokenWrapperError::UnexpectedFreezeAuthority,
        "Invalid freeze authority passed",
    )
}

pub fn assert_is_account_uninitialized(account: &AccountInfo) -> ProgramResult {
    let account_lamports = account.try_borrow_lamports()?;

    assert_with_msg(
        account.data_is_empty() && **account_lamports == 0u64,
        TokenWrapperError::UnexpectedInitializedAccount,
        "The account is already initialized, expected to be uninitialized",
    )
}

pub fn assert_is_account_initialized(account: &AccountInfo) -> ProgramResult {
    let account_lamports = account.try_borrow_lamports()?;

    assert_with_msg(
        **account_lamports > 0u64,
        TokenWrapperError::ExpectedInitializedAccount,
        "The account is not initialized, expected to be initialized",
    )
}

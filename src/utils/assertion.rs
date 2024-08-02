use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey, sysvar,
    system_program
};

use crate::error::TokenWrapperError;

use super::{get_reserve_authority, get_reserve_authority_token_account, get_wrapper_token_mint};

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

pub fn assert_wrapper_token_mint(
    token_2022_mint: Pubkey,
    program_id: Pubkey,
    actual_wrapper_token_mint: &AccountInfo,
    is_initialized: bool,
) -> ProgramResult {
    let (expected_wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, program_id);
    let expected_program_owner = if is_initialized {
        spl_token::id()
    } else {
        system_program::id()
    };

    assert_with_msg(
        &expected_wrapper_token_mint == actual_wrapper_token_mint.key && actual_wrapper_token_mint.owner == &expected_program_owner,
        TokenWrapperError::UnexpectedWrapperToken,
        "Invalid wrapper token mint passed",
    )
}

pub fn assert_reserve_authority(
    token_2022_mint: Pubkey,
    program_id: Pubkey,
    actual_reserve_authority: &AccountInfo,
) -> ProgramResult {
    let (expected_reserve_authority, _, _) = get_reserve_authority(token_2022_mint, program_id);

    assert_with_msg(
        &expected_reserve_authority == actual_reserve_authority.key,
        TokenWrapperError::UnexpectedReserveAuthority,
        "Invalid reserve authority passed",
    )
}

pub fn assert_reserve_authority_token_account(
    token_2022_mint: Pubkey,
    owner: Pubkey,
    program_id: Pubkey,
    actual_reserve_authority_token_account: &AccountInfo,
    is_initialized: bool
) -> ProgramResult {
    let (expected_reserve_authority_token_account, _, _) =
        get_reserve_authority_token_account(token_2022_mint, owner, program_id);
    let expected_program_owner = if is_initialized {
        spl_token_2022::id()
    } else {
        system_program::id()
    };

    assert_with_msg(
        &expected_reserve_authority_token_account == actual_reserve_authority_token_account.key && actual_reserve_authority_token_account.owner == &expected_program_owner,
        TokenWrapperError::UnexpectedReserveTokenAccount,
        "Invalid reserve authority token account passed",
    )
}

pub fn validate_token_account(
    token_account_info: &AccountInfo,
    expected_owner: &Pubkey,
    expected_mint: &Pubkey,
    is_token_2022: bool
) -> ProgramResult {
    let token_account_data = token_account_info.try_borrow_data()?;
    let token_account = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(&token_account_data)?;
    let expected_program_owner = if is_token_2022 {
        spl_token_2022::id()
    } else {
        spl_token::id()
    };

    assert_with_msg(
        &token_account.base.owner == expected_owner && token_account_info.owner == &expected_program_owner && &token_account.base.mint == expected_mint,
        TokenWrapperError::InvalidTokenAccount,
        "Incorrect token account"
    )
}

pub fn validate_mint(
    token_mint_info: &AccountInfo,
    is_token_2022: bool
) -> ProgramResult {
    let token_mint_data = token_mint_info.try_borrow_data()?;
    let token_mint = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    let expected_program_owner = if is_token_2022 {
        spl_token_2022::id()
    } else {
        spl_token::id()
    };

    assert_with_msg(
        token_mint.base.is_initialized && token_mint_info.owner == &expected_program_owner,
        TokenWrapperError::InvalidTokenMint,
        "Incorrect token mint"
    )    
}
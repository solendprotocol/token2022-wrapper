use solana_program::{entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey, sysvar};

use crate::error::TokenWrapperError;

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
    assert_with_msg(p == spl_token::ID, TokenWrapperError::UnexpectedTokenProgram, "Invalid Token program pubkey passed")
}

pub fn assert_token_2022_program(p: Pubkey) -> ProgramResult {
    assert_with_msg(p == spl_token_2022::ID, TokenWrapperError::UnexpectedTokenProgram, "Invalid Token 2022 program pubkey passed")
}

pub fn assert_system_program(p: Pubkey) -> ProgramResult {
    assert_with_msg(p == solana_program::system_program::ID, TokenWrapperError::UnexpectedTokenProgram, "Invalid System program pubkey passed")
}

pub fn assert_rent(p: Pubkey) -> ProgramResult {
    assert_with_msg(p == sysvar::rent::ID,TokenWrapperError::UnexpectedRent,  "Invalid Rent pubkey passed")
}
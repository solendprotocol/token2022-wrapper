use num_enum::IntoPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]

pub enum TokenWrapperError {
    #[error("Unexpected vanilla token mint")]
    UnexpectedVanillaToken = 0,
    #[error("Unexpected Token Program")]
    UnexpectedTokenProgram = 1,
    #[error("Unexpected Token 2022 Program")]
    UnexpectedToken2022Program = 2,
    #[error("Unexpected System Program")]
    UnexpectedSystemProgram = 3,
    #[error("Unexpected Rent")]
    UnexpectedRent = 4,
}

impl From<TokenWrapperError> for ProgramError {
    fn from(e: TokenWrapperError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

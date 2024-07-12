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
    #[error("Unexpected initialized account")]
    UnexpectedInitializedAccount = 5,
    #[error("Expected initialized account")]
    ExpectedInitializedAccount = 6,
    #[error("Unexpected user token account owner")]
    UnexpectedUserTokenAccountOwner = 7,
    #[error("Unexpected reserve token account owner")]
    UnexpectedReserveTokenAccountOwner = 8,
    #[error("Unexpected reserve authority")]
    UnexpectedReserveAuthority = 9,
    #[error("Unexpected mint authority")]
    UnexpectedMintAuthority = 10,
    #[error("Unexpected freeze authority")]
    UnexpectedFreezeAuthority = 11,
}

impl From<TokenWrapperError> for ProgramError {
    fn from(e: TokenWrapperError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
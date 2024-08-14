use num_enum::TryFromPrimitive;
use shank::ShankInstruction;
use solana_program::{msg, program_error::ProgramError};

#[repr(u8)]
#[derive(TryFromPrimitive, Clone, Copy, ShankInstruction, PartialEq, Eq)]
pub enum TokenWrapperInstruction {
    /// 0
    /// Initializes a wrapper token mint on the Token Program for a particular Token 2022 token
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[signer, writable]` The payer paying for the initialization of mint account on the Token program
    /// 1. `[]` Token2022 token mint
    /// 2. `[writable]` Wrapper token mint, uninitialized
    ///     Must be a PDA with seeds ["wrapper", Token2022 token mint]
    /// 3. `[writable]` Reserve authority, uninitialized
    ///     Must be a PDA with seeds ["reserve_authority", Token2022 token mint]
    /// 4. `[writable]` Reserve authority token account, uninitialized
    ///     Must be a PDA with seeds ["reserve_authority_token_account", Token2022 token mint, reserve_authority PDA pubkey]
    /// 3. `[]` SPL Token program
    /// 4. `[]` Token 2022 program
    /// 5. `[]` System program
    /// 6. `[]` Rent sysvar
    InitializeWrapperToken = 0,

    /// 1
    /// Mints wrapper tokens created using SPL Token Program in exchange of Token 2022 deposits
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[signer, writable]` User authority
    /// 1. `[]` Reserve authority
    ///     Must be a PDA with seeds ["reserve_authority", Token2022 token mint]
    /// 2. `[]` Token2022 token mint
    /// 3. `[writable]` Wrapper token mint
    /// 4. `[writable]` User's token account for the wrapper token
    /// 5. `[writable]` User's token account for the Token2022 token
    /// 6. `[writable]` Reserve's token account for the Token2022 token
    /// 7. `[]` SPL Token program
    /// 8. `[]` Token2022 program
    /// 9. `[]` System program
    /// 10. `[]` Associated token program
    /// 11. `[]` Rent sysvar
    DepositAndMintWrapperTokens = 1,

    /// 2
    /// Burns wrapper tokens created using Token Program in exchange of Token 2022 withdrawals
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[signer, writable]` User authority
    /// 1. `[]` Reserve authority
    ///     Must be a PDA with seeds ["reserve_authority", Token2022 token mint]
    /// 2. `[]` Token2022 token mint
    /// 3. `[]` Wrapper token mint
    /// 4. `[writable]` User's token account for the wrapper token
    /// 5. `[writable]` User's token account for the Token2022 token
    /// 6. `[writable]` Reserve's token account for the Token2022 token
    /// 7. `[]` SPL Token program
    /// 8. `[]` Token2022 program
    /// 9. `[]` System program
    /// 10. `[]` Rent sysvar
    WithdrawAndBurnWrapperTokens = 2,
}

impl TokenWrapperInstruction {
    // Unpacks a byte buffer into a valid TokenWrapperInstruction
    pub fn unpack(instruction_data: &[u8]) -> Result<Self, ProgramError> {
        let (tag, _) = instruction_data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => TokenWrapperInstruction::InitializeWrapperToken,
            1 => TokenWrapperInstruction::DepositAndMintWrapperTokens,
            2 => TokenWrapperInstruction::WithdrawAndBurnWrapperTokens,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    pub fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() < 8 {
            msg!("u64 cannot be unpacked");
            return Err(ProgramError::InvalidInstructionData);
        }
        let (bytes, rest) = input.split_at(8);
        let value = bytes
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok((value, rest))
    }

    pub fn unpack_u64_and_bool(input: &[u8]) -> Result<(u64, bool, &[u8]), ProgramError> {
        if input.len() < 9 {
            msg!("u64 and bool cannot be unpacked");
            return Err(ProgramError::InvalidInstructionData);
        }
        let (u64_bytes, rest) = input.split_at(8);
        let (bool_byte, rest) = rest.split_first().unwrap();

        let u64_value = u64_bytes
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;

        let bool_value = *bool_byte != 0;

        Ok((u64_value, bool_value, rest))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

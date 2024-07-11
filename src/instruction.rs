use solana_program::{msg, program_error::ProgramError};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TokenWrapperInstruction {
    /// 0
    /// Initializes a vanilla token mint on the Token Program for a particular Token 2022 token
    /// 
    /// Accounts expected by this instruction:
    /// 
    /// 0. `[signer]` The payer paying for the initialization of mint account on the Token program
    /// 1. `[]` Token2022 token mint
    /// 2. `[writable]` Vanilla token mint, uninitialized
    ///     Must be a PDA with seeds ["vanilla", Token2022 token mint]
    /// 3. `[]` Vanilla Token program
    /// 4. `[]` System program
    /// 5. `[]` Rent sysvar
    InitializeToken,

    /// 1
    /// Mints vanilla tokens created using Token Program in exchange of Token 2022 deposits
    /// 
    /// Accounts expected by this instruction:
    /// 
    /// 0. `[signer]` User authority
    /// 1. `[]` Token2022 token mint
    /// 2. `[]` Vanilla token mint
    /// 3. `[writable]` User's token account for the vanilla token
    /// 4. `[writable]` User's token account for the Token2022 token
    /// 5. `[writable]` Reserve's token account for the Token2022 token
    ///     Must be a PDA with seeds ["reserve", Token2022 token mint, User's authority pubkey]
    /// 6. `[]` Vanilla Token program
    /// 7. `[]` Token2022 program
    /// 8. `[]` System program
    DepositAndMintTokens {
        amount: u64
    },

    /// 2
    /// Burns vanilla tokens created using Token Program in exchange of Token 2022 withdrawals
    /// 
    /// Accounts expected by this instruction:
    /// 
    /// 0. `[signer]` User authority
    /// 1. `[]` Token2022 token mint
    /// 2. `[]` Vanilla token mint
    /// 3. `[writable]` User's token account for the vanilla token
    /// 4. `[writable]` User's token account for the Token2022 token
    /// 5. `[writable]` Reserve's token account for the Token2022 token
    ///     Must be a PDA with seeds ["reserve", Token2022 token mint, User's authority pubkey]
    /// 6. `[]` Vanilla Token program
    /// 7. `[]` Token2022 program
    /// 8. `[]` System program
    WithdrawAndBurnTokens {
        amount: u64
    },
}

impl TokenWrapperInstruction {
    // Unpacks a byte buffer into a valid TokenWrapperInstruction
    pub fn unpack(instruction_data: &[u8]) -> Result<Self, ProgramError> {
        let (tag, data) = instruction_data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(
            match tag {
                0 => {
                    TokenWrapperInstruction::InitializeToken
                }
                1 => {
                   let (amount, _rest) = Self::unpack_u64(data)?;
                    TokenWrapperInstruction::DepositAndMintTokens { amount }
                }
                2 => {
                    let (amount, _rest) = Self::unpack_u64(data)?;
                    TokenWrapperInstruction::WithdrawAndBurnTokens { amount }
                }
                _ => return Err(ProgramError::InvalidInstructionData)
            }
        )
    }

    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
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
}
use solana_program::program_error::ProgramError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TokenWrapperInstruction {
    /// 0
    /// Initializes a vanilla token mint on the Token Program for a particular Token 2022 token
    /// 
    /// Accounts expected by this instruction:
    /// 
    /// 0. `[signer]` Only admin with the authority can call this instruction
    /// 1. `[]` Token2022 token mint
    /// 2. `[writable]` Vanilla token mint, uninitialized
    ///     Must be a PDA with seeds ["vanilla", Token2022 token mint]
    /// 3. `[]` Vanilla Token program
    /// 4. `[]` System program
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
                _ => {
                    TokenWrapperInstruction::InitializeToken
                }
            }
        )
    }
}
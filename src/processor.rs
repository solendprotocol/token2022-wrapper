use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey
};

use crate::instruction::TokenWrapperInstruction;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    let instruction = TokenWrapperInstruction::unpack(instruction_data)?;

    match instruction {
        TokenWrapperInstruction::InitializeToken => {
            process_initialize_token(program_id, accounts)
        },
        TokenWrapperInstruction::DepositAndMintTokens { amount } => {
            process_deposit_and_mint_tokens(program_id, accounts, amount)
        },
        TokenWrapperInstruction::WithdrawAndBurnTokens { amount } => {
            process_withdraw_and_burn_tokens(program_id, accounts, amount)
        }
    }
}

pub fn process_initialize_token (
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> ProgramResult {
    msg!("TokenWrapperInstruction::InitializeToken");

    Ok(())
}

pub fn process_deposit_and_mint_tokens (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64
) -> ProgramResult {
    msg!("TokenWrapperInstruction::DepositAndMintTokens");

    Ok(())
}

pub fn process_withdraw_and_burn_tokens (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64
) -> ProgramResult {
    msg!("TokenWrapperInstruction::WithdrawAndBurnTokens");

    Ok(())
}
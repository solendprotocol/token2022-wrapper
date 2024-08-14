use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    instruction::TokenWrapperInstruction,
    utils::{get_reserve_authority, get_reserve_authority_token_account, get_wrapper_token_mint},
};

pub fn create_initialize_wrapper_token_instruction(
    payer: &Pubkey,
    token_2022_mint: &Pubkey,
) -> Instruction {
    let (wrapper_token_mint, _, _) = get_wrapper_token_mint(*token_2022_mint, crate::id());
    let (reserve_authority, _, _) = get_reserve_authority(*token_2022_mint, crate::id());
    let (reserve_token_2022_token_account, _, _) =
        get_reserve_authority_token_account(*token_2022_mint, reserve_authority, crate::id());

    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*token_2022_mint, false),
            AccountMeta::new(wrapper_token_mint, false),
            AccountMeta::new(reserve_authority, false),
            AccountMeta::new(reserve_token_2022_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: TokenWrapperInstruction::InitializeWrapperToken.to_vec(),
    }
}

pub fn create_deposit_and_mint_wrapper_tokens_instruction(
    user_authority: &Pubkey,
    token_2022_mint: &Pubkey,
    user_wrapper_token_account: &Pubkey,
    user_token_2022_token_account: &Pubkey,
    amount: u64,
    use_max_amount: bool
) -> Instruction {
    let (wrapper_token_mint, _, _) = get_wrapper_token_mint(*token_2022_mint, crate::id());
    let (reserve_authority, _, _) = get_reserve_authority(*token_2022_mint, crate::id());

    let (reserve_token_2022_token_account, _, _) =
        get_reserve_authority_token_account(*token_2022_mint, reserve_authority, crate::id());

    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&TokenWrapperInstruction::DepositAndMintWrapperTokens.to_vec());
    instruction_data.extend_from_slice(&amount.to_le_bytes());
    instruction_data.push(use_max_amount as u8);

    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(*user_authority, true),
            AccountMeta::new_readonly(reserve_authority, false),
            AccountMeta::new_readonly(*token_2022_mint, false),
            AccountMeta::new(wrapper_token_mint, false),
            AccountMeta::new(*user_wrapper_token_account, false),
            AccountMeta::new(*user_token_2022_token_account, false),
            AccountMeta::new(reserve_token_2022_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: instruction_data,
    }
}

pub fn create_withdraw_and_burn_wrapper_tokens_instruction(
    user_authority: &Pubkey,
    token_2022_mint: &Pubkey,
    user_wrapper_token_account: &Pubkey,
    user_token_2022_token_account: &Pubkey,
    amount: u64,
    use_max_amount: bool
) -> Instruction {
    let (wrapper_token_mint, _, _) = get_wrapper_token_mint(*token_2022_mint, crate::id());
    let (reserve_authority, _, _) = get_reserve_authority(*token_2022_mint, crate::id());

    let (reserve_token_2022_token_account, _, _) =
        get_reserve_authority_token_account(*token_2022_mint, reserve_authority, crate::id());

    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&TokenWrapperInstruction::WithdrawAndBurnWrapperTokens.to_vec());
    instruction_data.extend_from_slice(&amount.to_le_bytes());
    instruction_data.push(use_max_amount as u8);

    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(*user_authority, true),
            AccountMeta::new_readonly(reserve_authority, false),
            AccountMeta::new_readonly(*token_2022_mint, false),
            AccountMeta::new(wrapper_token_mint, false),
            AccountMeta::new(*user_wrapper_token_account, false),
            AccountMeta::new(*user_token_2022_token_account, false),
            AccountMeta::new(reserve_token_2022_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: instruction_data,
    }
}

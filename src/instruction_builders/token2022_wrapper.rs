use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{instruction::TokenWrapperInstruction, utils::{get_reserve_authority, get_token_mint_authority, get_vanilla_token_mint}};


pub fn create_initialize_token_instruction(
    payer: &Pubkey,
    token_2022_mint: &Pubkey,
) -> Instruction {
    let (vanilla_token_mint, _, _) = get_vanilla_token_mint(*token_2022_mint, crate::id());
    let (reserve_authority, _, _) = get_reserve_authority(*token_2022_mint, crate::id());
    let reserve_token_2022_token_account = get_associated_token_address(&reserve_authority, &token_2022_mint);

    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*token_2022_mint, false),
            AccountMeta::new(vanilla_token_mint, false),
            AccountMeta::new(reserve_authority, false),
            AccountMeta::new(reserve_token_2022_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: TokenWrapperInstruction::InitializeToken.to_vec(),
    }
}

pub fn create_deposit_and_mint_instruction(
    user_authority: &Pubkey,
    token_2022_mint: &Pubkey,
    user_vanilla_token_account: &Pubkey,
    user_token_2022_token_account: &Pubkey,
    amount: u64
) -> Instruction {
    let (vanilla_token_mint, _, _) = get_vanilla_token_mint(*token_2022_mint, crate::id());
    let (reserve_authority, _, _) = get_reserve_authority(*token_2022_mint, crate::id());
    let (mint_authority, _, _) = get_token_mint_authority(vanilla_token_mint, crate::id());

    let reserve_token_2022_token_account = get_associated_token_address(&reserve_authority, &token_2022_mint);

    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(*user_authority, true),
            AccountMeta::new_readonly(reserve_authority, false),
            AccountMeta::new_readonly(mint_authority, false),
            AccountMeta::new_readonly(*token_2022_mint, false),
            AccountMeta::new_readonly(vanilla_token_mint, false),
            AccountMeta::new(*user_vanilla_token_account, false),
            AccountMeta::new(*user_token_2022_token_account, false),
            AccountMeta::new(reserve_token_2022_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(system_program::id(), false)
        ],
        data: [
            TokenWrapperInstruction::DepositAndMintTokens.to_vec(),
            amount.to_le_bytes().to_vec()
        ].concat()
    }
}

pub fn create_withdraw_and_burn_instruction(
    user_authority: &Pubkey,
    token_2022_mint: &Pubkey,
    user_vanilla_token_account: &Pubkey,
    user_token_2022_token_account: &Pubkey,
    amount: u64
) -> Instruction {
    let (vanilla_token_mint, _, _) = get_vanilla_token_mint(*token_2022_mint, crate::id());
    let (reserve_authority, _, _) = get_reserve_authority(*token_2022_mint, crate::id());
    let (mint_authority, _, _) = get_token_mint_authority(vanilla_token_mint, crate::id());

    let reserve_token_2022_token_account = get_associated_token_address(&reserve_authority, &token_2022_mint);

    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(*user_authority, true),
            AccountMeta::new_readonly(reserve_authority, false),
            AccountMeta::new_readonly(mint_authority, false),
            AccountMeta::new_readonly(*token_2022_mint, false),
            AccountMeta::new_readonly(vanilla_token_mint, false),
            AccountMeta::new(*user_vanilla_token_account, false),
            AccountMeta::new(*user_token_2022_token_account, false),
            AccountMeta::new(reserve_token_2022_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(system_program::id(), false)
        ],
        data: [
            TokenWrapperInstruction::DepositAndMintTokens.to_vec(),
            amount.to_le_bytes().to_vec()
        ].concat()
    }
}
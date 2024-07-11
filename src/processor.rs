use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_program,
    sysvar::Sysvar,
};

use crate::{
    error::TokenWrapperError,
    instruction::TokenWrapperInstruction,
    utils::{
        assert_rent, assert_system_program, assert_token_program, assert_with_msg,
        get_token_freeze_authority, get_token_mint_authority, get_vanilla_token_mint,
    },
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TokenWrapperInstruction::unpack(instruction_data)?;

    match instruction {
        TokenWrapperInstruction::InitializeToken => process_initialize_token(program_id, accounts),
        TokenWrapperInstruction::DepositAndMintTokens { amount } => {
            process_deposit_and_mint_tokens(program_id, accounts, amount)
        }
        TokenWrapperInstruction::WithdrawAndBurnTokens { amount } => {
            process_withdraw_and_burn_tokens(program_id, accounts, amount)
        }
    }
}

pub fn process_initialize_token(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    msg!("TokenWrapperInstruction::InitializeToken");

    let accounts_info_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_info_iter)?;
    let token_2022_mint = next_account_info(accounts_info_iter)?;
    let vanilla_token_mint = next_account_info(accounts_info_iter)?;
    let token_program = next_account_info(accounts_info_iter)?;
    let system_program = next_account_info(accounts_info_iter)?;
    let rent_sysvar = next_account_info(accounts_info_iter)?;

    let (expected_vanilla_token_mint, _, _) =
        get_vanilla_token_mint(*token_2022_mint.key, *program_id);
    let (mint_authority, _, _) = get_token_mint_authority(*vanilla_token_mint.key, *program_id);
    let (freeze_authority, _, _) = get_token_freeze_authority(*vanilla_token_mint.key, *program_id);

    let unwrapped_token_2022_mint_data = token_2022_mint.try_borrow_data()?;
    let token_2022_mint_data =
        spl_token_2022::state::Mint::unpack(&unwrapped_token_2022_mint_data)?;

    assert_with_msg(
        expected_vanilla_token_mint == *vanilla_token_mint.key,
        TokenWrapperError::UnexpectedVanillaToken,
        "Invalid vanilla token mint passed",
    )?;
    assert_token_program(*token_program.key)?;
    assert_system_program(*system_program.key)?;
    assert_rent(*rent_sysvar.key)?;

    let init_mint_ix = spl_token::instruction::initialize_mint(
        token_program.key,
        vanilla_token_mint.key,
        &mint_authority,
        Some(&freeze_authority),
        token_2022_mint_data.decimals,
    )?;

    invoke(
        &init_mint_ix,
        &[
            vanilla_token_mint.clone(),
            rent_sysvar.clone(),
            token_program.clone(),
            payer.clone(),
        ],
    )?;

    Ok(())
}

pub fn process_deposit_and_mint_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("TokenWrapperInstruction::DepositAndMintTokens");

    Ok(())
}

pub fn process_withdraw_and_burn_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("TokenWrapperInstruction::WithdrawAndBurnTokens");

    Ok(())
}

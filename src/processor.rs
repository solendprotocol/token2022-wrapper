use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{
    system_program,
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_associated_token_account::tools::account::get_account_len;
use spl_token::state::Mint;
use spl_token_2022::extension::ExtensionType;

use crate::error::TokenWrapperError;
use crate::utils::{assert_with_msg, create_account, validate_mint, validate_token_account};
use crate::{
    instruction::TokenWrapperInstruction,
    utils::{
        assert_rent,
        assert_reserve_authority, assert_reserve_authority_token_account, assert_system_program,
        assert_token_2022_program, assert_token_program,
        assert_wrapper_token_mint, get_reserve_authority, get_reserve_authority_token_account,
        get_wrapper_token_mint,
    },
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TokenWrapperInstruction::unpack(instruction_data)?;

    let (_, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match instruction {
        TokenWrapperInstruction::InitializeWrapperToken => {
            process_initialize_wrapper_token(program_id, accounts)
        }
        TokenWrapperInstruction::DepositAndMintWrapperTokens => {
            let (amount, _) = TokenWrapperInstruction::unpack_u64(data)?;

            process_deposit_and_mint_wrapper_tokens(program_id, accounts, amount)
        }
        TokenWrapperInstruction::WithdrawAndBurnWrapperTokens => {
            let (amount, _) = TokenWrapperInstruction::unpack_u64(data)?;

            process_withdraw_and_burn_wrapper_tokens(program_id, accounts, amount)
        }
    }
}

pub fn process_initialize_wrapper_token(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("TokenWrapperInstruction::InitializeWrapperToken");

    let accounts_info_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_info_iter)?;
    let token_2022_mint = next_account_info(accounts_info_iter)?;
    let wrapper_token_mint = next_account_info(accounts_info_iter)?;
    let reserve_authority = next_account_info(accounts_info_iter)?;
    let reserve_token_2022_token_account = next_account_info(accounts_info_iter)?;
    let token_program = next_account_info(accounts_info_iter)?;
    let token_2022_program = next_account_info(accounts_info_iter)?;
    let system_program = next_account_info(accounts_info_iter)?;
    let rent_sysvar = next_account_info(accounts_info_iter)?;

    assert_with_msg(payer.is_signer, TokenWrapperError::MissingSigner, "The payer account needs to be a signer")?;

    assert_wrapper_token_mint(*token_2022_mint.key, *program_id, wrapper_token_mint, false)?;
    assert_reserve_authority(*token_2022_mint.key, *program_id, reserve_authority)?;
    assert_reserve_authority_token_account(
        *token_2022_mint.key,
        *reserve_authority.key,
        *program_id,
        reserve_token_2022_token_account,
        false
    )?;

    assert_token_program(*token_program.key)?;
    assert_system_program(*system_program.key)?;
    assert_rent(*rent_sysvar.key)?;

    validate_mint(token_2022_mint, true)?;

    let (_, _, wrapper_token_mint_seeds) =
        get_wrapper_token_mint(*token_2022_mint.key, *program_id);

    let (_, _, reserve_authority_seeds) = get_reserve_authority(*token_2022_mint.key, *program_id);

    let (_, _, reserve_token_account_seeds) = get_reserve_authority_token_account(
        *token_2022_mint.key,
        *reserve_authority.key,
        *program_id,
    );

    let mint_data_length = Mint::LEN as u64;
    let rent = Rent::get().unwrap();

    create_account(
        &payer, 
        &wrapper_token_mint,
        system_program,
        &spl_token::id(),
        &rent,
        mint_data_length,
        wrapper_token_mint_seeds.clone()
    )?;

    let token_2022_mint_data = token_2022_mint.try_borrow_data()?;
    let token_2022_mint_data_parsed = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_2022_mint_data)?;
    let token_2022_decimals = token_2022_mint_data_parsed.base.decimals;
    drop(token_2022_mint_data);

    let init_mint_ix = spl_token::instruction::initialize_mint(
        token_program.key,
        wrapper_token_mint.key,
        &reserve_authority.key,
        Some(&reserve_authority.key),
        token_2022_decimals,
    )?;

    invoke_signed(
        &init_mint_ix,
        &[
            wrapper_token_mint.clone(),
            rent_sysvar.clone(),
            token_program.clone(),
            payer.clone(),
        ],
        &[wrapper_token_mint_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    let token_account_data_length = get_account_len(
        &token_2022_mint.clone(),
        &token_2022_program.clone(),
        &[ExtensionType::ImmutableOwner],
    )? as u64;
    let rent = Rent::get().unwrap();

    create_account(
        &payer, 
        &reserve_token_2022_token_account,
        system_program,
        &spl_token_2022::id(),
        &rent,
        token_account_data_length,
        reserve_token_account_seeds.clone()
    )?;

    invoke(
        &spl_token_2022::instruction::initialize_immutable_owner(
            &spl_token_2022::id(),
            reserve_token_2022_token_account.key,
        )?,
        &[
            reserve_token_2022_token_account.clone(),
            token_2022_program.clone(),
        ],
    )?;

    invoke_signed(
        &spl_token_2022::instruction::initialize_account3(
            &spl_token_2022::id(),
            reserve_token_2022_token_account.key,
            token_2022_mint.key,
            reserve_authority.key,
        )?,
        &[
            reserve_token_2022_token_account.clone(),
            token_2022_mint.clone(),
            reserve_authority.clone(),
            token_2022_program.clone(),
        ],
        &[reserve_authority_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    msg!("TokenWrapperInstruction::InitializeWrapperToken --> Everything done, returning");

    Ok(())
}

pub fn process_deposit_and_mint_wrapper_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("TokenWrapperInstruction::DepositAndMintWrapperTokens");

    let accounts_info_iter = &mut accounts.iter();
    let user_authority = next_account_info(accounts_info_iter)?;
    let reserve_authority = next_account_info(accounts_info_iter)?;
    let token_2022_mint = next_account_info(accounts_info_iter)?;
    let wrapper_token_mint = next_account_info(accounts_info_iter)?;
    let user_wrapper_token_account = next_account_info(accounts_info_iter)?;
    let user_token_2022_token_account = next_account_info(accounts_info_iter)?;
    let reserve_token_2022_token_account = next_account_info(accounts_info_iter)?;
    let token_program = next_account_info(accounts_info_iter)?;
    let token_2022_program = next_account_info(accounts_info_iter)?;
    let system_program = next_account_info(accounts_info_iter)?;
    let associated_token_program = next_account_info(accounts_info_iter)?;
    let rent_sysvar = next_account_info(accounts_info_iter)?;

    assert_with_msg(user_authority.is_signer, TokenWrapperError::MissingSigner, "The user authority needs to be a signer")?;

    assert_wrapper_token_mint(*token_2022_mint.key, *program_id, wrapper_token_mint, true)?;
    assert_reserve_authority(*token_2022_mint.key, *program_id, reserve_authority)?;
    assert_reserve_authority_token_account(
        *token_2022_mint.key,
        *reserve_authority.key,
        *program_id,
        reserve_token_2022_token_account,
        true
    )?;

    assert_token_program(*token_program.key)?;
    assert_token_2022_program(*token_2022_program.key)?;
    assert_system_program(*system_program.key)?;
    assert_rent(*rent_sysvar.key)?;

    let token_2022_mint_data = token_2022_mint.try_borrow_data()?;
    let token_2022_mint_data_parsed = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_2022_mint_data)?;
    let token_2022_decimals = token_2022_mint_data_parsed.base.decimals;
    drop(token_2022_mint_data);

    if user_wrapper_token_account.owner == &system_program::id() {
        let ata_init_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                user_authority.key,
                user_authority.key,
                wrapper_token_mint.key,
                token_program.key,
            );

        invoke(
            &ata_init_ix,
            &[
                user_authority.clone(),
                user_wrapper_token_account.clone(),
                wrapper_token_mint.clone(),
                system_program.clone(),
                token_program.clone(),
                associated_token_program.clone(),
            ],
        )?;
    }

    validate_mint(token_2022_mint, true)?;
    validate_mint(wrapper_token_mint, false)?;

    validate_token_account(user_token_2022_token_account, user_authority.key, token_2022_mint.key, true)?;
    validate_token_account(user_wrapper_token_account, user_authority.key, wrapper_token_mint.key, false)?;
    validate_token_account(reserve_token_2022_token_account, reserve_authority.key, token_2022_mint.key, true)?;

    let reserve_token_2022_token_account_data = reserve_token_2022_token_account.try_borrow_data()?;
    let reserve_token_2022_data_parsed = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(&reserve_token_2022_token_account_data)?;
    let pre_transfer_balance = reserve_token_2022_data_parsed.base.amount;
    drop(reserve_token_2022_token_account_data);

    let user_deposit_ix = spl_token_2022::instruction::transfer_checked(
        token_2022_program.key,
        user_token_2022_token_account.key,
        token_2022_mint.key,
        reserve_token_2022_token_account.key,
        user_authority.key,
        &[user_authority.key],
        amount,
        token_2022_decimals,
    )?;

    invoke(
        &user_deposit_ix,
        &[
            token_2022_program.clone(),
            user_token_2022_token_account.clone(),
            token_2022_mint.clone(),
            reserve_token_2022_token_account.clone(),
            user_authority.clone(),
        ],
    )?;

    let reserve_token_2022_token_account_data = reserve_token_2022_token_account.try_borrow_data()?;
    let reserve_token_2022_data_parsed = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Account>::unpack(&reserve_token_2022_token_account_data)?;
    let post_transfer_balance = reserve_token_2022_data_parsed.base.amount;
    drop(reserve_token_2022_token_account_data);

    let (_, _, reserve_authority_seeds) = get_reserve_authority(*token_2022_mint.key, *program_id);

    let user_mint_ix = spl_token::instruction::mint_to_checked(
        token_program.key,
        wrapper_token_mint.key,
        user_wrapper_token_account.key,
        reserve_authority.key,
        &[reserve_authority.key],
        post_transfer_balance.checked_sub(pre_transfer_balance).unwrap(),
        token_2022_decimals,
    )?;

    invoke_signed(
        &user_mint_ix,
        &[
            token_program.clone(),
            wrapper_token_mint.clone(),
            user_wrapper_token_account.clone(),
            reserve_authority.clone(),
        ],
        &[reserve_authority_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    msg!("TokenWrapperInstruction::DepositAndMintWrapperTokens --> Everything done, returning");

    Ok(())
}

pub fn process_withdraw_and_burn_wrapper_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("TokenWrapperInstruction::WithdrawAndBurnWrapperTokens");

    let accounts_info_iter = &mut accounts.iter();
    let user_authority = next_account_info(accounts_info_iter)?;
    let reserve_authority = next_account_info(accounts_info_iter)?;
    let token_2022_mint = next_account_info(accounts_info_iter)?;
    let wrapper_token_mint = next_account_info(accounts_info_iter)?;
    let user_wrapper_token_account = next_account_info(accounts_info_iter)?;
    let user_token_2022_token_account = next_account_info(accounts_info_iter)?;
    let reserve_token_2022_token_account = next_account_info(accounts_info_iter)?;
    let token_program = next_account_info(accounts_info_iter)?;
    let token_2022_program = next_account_info(accounts_info_iter)?;
    let system_program = next_account_info(accounts_info_iter)?;
    let rent_sysvar = next_account_info(accounts_info_iter)?;

    assert_with_msg(user_authority.is_signer, TokenWrapperError::MissingSigner, "The user authority needs to be a signer")?;

    assert_wrapper_token_mint(*token_2022_mint.key, *program_id, wrapper_token_mint, true)?;
    assert_reserve_authority(*token_2022_mint.key, *program_id, reserve_authority)?;
    assert_reserve_authority_token_account(
        *token_2022_mint.key,
        *reserve_authority.key,
        *program_id,
        reserve_token_2022_token_account,
        true
    )?;

    assert_token_program(*token_program.key)?;
    assert_token_2022_program(*token_2022_program.key)?;
    assert_system_program(*system_program.key)?;
    assert_rent(*rent_sysvar.key)?;

    validate_mint(token_2022_mint, true)?;
    validate_mint(wrapper_token_mint, false)?;

    validate_token_account(user_token_2022_token_account, user_authority.key, token_2022_mint.key, true)?;
    validate_token_account(user_wrapper_token_account, user_authority.key, wrapper_token_mint.key, false)?;
    validate_token_account(reserve_token_2022_token_account, reserve_authority.key, token_2022_mint.key, true)?;

    let token_2022_mint_data = token_2022_mint.try_borrow_data()?;
    let token_2022_mint_data_parsed = spl_token_2022::extension::StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_2022_mint_data)?;
    let token_2022_decimals = token_2022_mint_data_parsed.base.decimals;
    drop(token_2022_mint_data);

    let user_burn_ix = spl_token::instruction::burn_checked(
        token_program.key,
        user_wrapper_token_account.key,
        wrapper_token_mint.key,
        user_authority.key,
        &[user_authority.key],
        amount,
        token_2022_decimals,
    )?;

    invoke(
        &user_burn_ix,
        &[
            token_program.clone(),
            user_wrapper_token_account.clone(),
            wrapper_token_mint.clone(),
            user_authority.clone(),
        ],
    )?;

    let user_withdraw_ix = spl_token_2022::instruction::transfer_checked(
        token_2022_program.key,
        reserve_token_2022_token_account.key,
        token_2022_mint.key,
        user_token_2022_token_account.key,
        reserve_authority.key,
        &[reserve_authority.key],
        amount,
        token_2022_decimals,
    )?;

    let (_, _, reserve_authority_seeds) = get_reserve_authority(*token_2022_mint.key, *program_id);

    invoke_signed(
        &user_withdraw_ix,
        &[
            token_2022_program.clone(),
            reserve_token_2022_token_account.clone(),
            token_2022_mint.clone(),
            user_token_2022_token_account.clone(),
            reserve_authority.clone(),
        ],
        &[reserve_authority_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    msg!("TokenWrapperInstruction::WithdrawAndBurnWrapperTokens --> Everything done, returning");

    Ok(())
}

use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use solana_program::system_instruction;
use spl_associated_token_account::tools::account::get_account_len;
use spl_token::state::Mint;
use spl_token_2022::extension::ExtensionType;

use crate::{
    error::TokenWrapperError,
    instruction::TokenWrapperInstruction,
    utils::{
        assert_is_account_initialized, assert_is_account_uninitialized, assert_mint_authority,
        assert_rent, assert_reserve_authority, assert_reserve_authority_token_account,
        assert_system_program, assert_token_2022_program, assert_token_program, assert_with_msg,
        assert_wrapper_token_mint, get_reserve_authority, get_reserve_authority_token_account,
        get_token_freeze_authority, get_token_mint_authority, get_wrapper_token_mint,
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

    let (mint_authority, _, _) = get_token_mint_authority(*wrapper_token_mint.key, *program_id);
    let (freeze_authority, _, _) = get_token_freeze_authority(*wrapper_token_mint.key, *program_id);

    let unwrapped_token_2022_mint_data = token_2022_mint.try_borrow_data()?;
    let token_2022_mint_data =
        spl_token_2022::state::Mint::unpack(&unwrapped_token_2022_mint_data)?;

    assert_wrapper_token_mint(*token_2022_mint.key, *program_id, *wrapper_token_mint.key)?;
    assert_reserve_authority(*token_2022_mint.key, *program_id, *reserve_authority.key)?;
    assert_reserve_authority_token_account(
        *token_2022_mint.key,
        *reserve_authority.key,
        *program_id,
        *reserve_token_2022_token_account.key,
    )?;
    assert_is_account_uninitialized(wrapper_token_mint)?;
    assert_is_account_uninitialized(reserve_authority)?;
    assert_is_account_uninitialized(reserve_token_2022_token_account)?;

    assert_token_program(*token_program.key)?;
    assert_system_program(*system_program.key)?;
    assert_rent(*rent_sysvar.key)?;

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
    let mint_lamports = rent.minimum_balance(mint_data_length as usize);

    let create_mint_account_ix = system_instruction::create_account(
        payer.key, 
        wrapper_token_mint.key, 
        mint_lamports, 
        mint_data_length, 
        &spl_token::id()
    );

    invoke_signed(
        &create_mint_account_ix,
        &[
            payer.clone(),
            wrapper_token_mint.clone(),
            system_program.clone(),
        ],
        &[wrapper_token_mint_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    let init_mint_ix = spl_token::instruction::initialize_mint(
        token_program.key,
        wrapper_token_mint.key,
        &mint_authority,
        Some(&freeze_authority),
        token_2022_mint_data.decimals,
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
    let token_account_lamports = rent.minimum_balance(token_account_data_length as usize);

    let create_reserve_token_account_ix = system_instruction::create_account(
        payer.key, 
        reserve_token_2022_token_account.key, 
        token_account_lamports, 
        token_account_data_length, 
        &spl_token_2022::id()
    );

    invoke_signed(
        &create_reserve_token_account_ix,
        &[
            payer.clone(),
            reserve_token_2022_token_account.clone(),
            system_program.clone(),
        ],
        &[reserve_token_account_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
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
    let mint_authority = next_account_info(accounts_info_iter)?;
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

    assert_wrapper_token_mint(*token_2022_mint.key, *program_id, *wrapper_token_mint.key)?;
    assert_is_account_initialized(wrapper_token_mint)?;
    assert_reserve_authority(*token_2022_mint.key, *program_id, *reserve_authority.key)?;
    assert_reserve_authority_token_account(
        *token_2022_mint.key,
        *reserve_authority.key,
        *program_id,
        *reserve_token_2022_token_account.key,
    )?;
    assert_mint_authority(*wrapper_token_mint.key, *program_id, *mint_authority.key)?;

    assert_token_program(*token_program.key)?;
    assert_token_2022_program(*token_2022_program.key)?;
    assert_system_program(*system_program.key)?;
    assert_rent(*rent_sysvar.key)?;

    let unwrapped_token_2022_mint_data = token_2022_mint.try_borrow_data()?;
    let token_2022_mint_data =
        spl_token_2022::state::Mint::unpack(&unwrapped_token_2022_mint_data)?;

    if user_wrapper_token_account.lamports() == 0 {
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

    let user_token_2022_token_account_data = user_token_2022_token_account.try_borrow_data()?;
    let (user_token_2022_token_account_data_stripped, _) = user_token_2022_token_account_data.split_at(spl_token::state::Account::LEN);
    let user_token_2022_data = spl_token_2022::state::Account::unpack(&user_token_2022_token_account_data_stripped).unwrap();
    drop(user_token_2022_token_account_data);
    assert_with_msg(
        &user_token_2022_data.owner == user_authority.key,
        TokenWrapperError::UnexpectedUserTokenAccountOwner,
        "User does not own the token account for this Token 2022 token",
    )?;

    let user_wrapper_token_account_data = user_wrapper_token_account.try_borrow_data()?;
    let (user_wrapper_token_account_data_stripped, _) = user_wrapper_token_account_data.split_at(spl_token::state::Account::LEN);
    let user_wrapper_token_data = spl_token::state::Account::unpack(&user_wrapper_token_account_data_stripped).unwrap();
    drop(user_wrapper_token_account_data);
    assert_with_msg(
        &user_wrapper_token_data.owner == user_authority.key,
        TokenWrapperError::UnexpectedUserTokenAccountOwner,
        "User does not own the token account for the wrapper token",
    )?;

    let reserve_token_2022_token_account_data = reserve_token_2022_token_account.try_borrow_data()?;
    let (reserve_token_2022_token_account_data_stripped, _) = reserve_token_2022_token_account_data.split_at(spl_token::state::Account::LEN);
    let reserve_token_2022_data = spl_token_2022::state::Account::unpack(&reserve_token_2022_token_account_data_stripped).unwrap();
    drop(reserve_token_2022_token_account_data);
    assert_with_msg(
        &reserve_token_2022_data.owner == reserve_authority.key,
        TokenWrapperError::UnexpectedReserveTokenAccountOwner,
        "The reserve does not own the token account for this Token 2022 token",
    )?;

    let pre_transfer_balance = reserve_token_2022_data.amount;

    let user_deposit_ix = spl_token_2022::instruction::transfer_checked(
        token_2022_program.key,
        user_token_2022_token_account.key,
        token_2022_mint.key,
        reserve_token_2022_token_account.key,
        user_authority.key,
        &[user_authority.key],
        amount,
        token_2022_mint_data.decimals,
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

    let reserve_token_account_data_copy = reserve_token_2022_token_account.try_borrow_data()?;
    let (reserve_token_account_data_copy_stripped, _) = reserve_token_account_data_copy.split_at(spl_token::state::Account::LEN);
    let post_transfer_balance = spl_token_2022::state::Account::unpack(&reserve_token_account_data_copy_stripped)?.amount;
    drop(reserve_token_account_data_copy);

    let (_, _, mint_authority_seeds) =
        get_token_mint_authority(*wrapper_token_mint.key, *program_id);

    let user_mint_ix = spl_token::instruction::mint_to_checked(
        token_program.key,
        wrapper_token_mint.key,
        user_wrapper_token_account.key,
        mint_authority.key,
        &[mint_authority.key],
        post_transfer_balance - pre_transfer_balance,
        token_2022_mint_data.decimals,
    )?;

    invoke_signed(
        &user_mint_ix,
        &[
            token_program.clone(),
            wrapper_token_mint.clone(),
            user_wrapper_token_account.clone(),
            mint_authority.clone(),
        ],
        &[mint_authority_seeds
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

    assert_wrapper_token_mint(*token_2022_mint.key, *program_id, *wrapper_token_mint.key)?;
    assert_is_account_initialized(wrapper_token_mint)?;
    assert_reserve_authority(*token_2022_mint.key, *program_id, *reserve_authority.key)?;
    assert_reserve_authority_token_account(
        *token_2022_mint.key,
        *reserve_authority.key,
        *program_id,
        *reserve_token_2022_token_account.key,
    )?;

    assert_token_program(*token_program.key)?;
    assert_token_2022_program(*token_2022_program.key)?;
    assert_system_program(*system_program.key)?;
    assert_rent(*rent_sysvar.key)?;

    let unwrapped_token_2022_mint_data = token_2022_mint.try_borrow_data()?;
    let token_2022_mint_data =
        spl_token_2022::state::Mint::unpack(&unwrapped_token_2022_mint_data)?;

    assert_with_msg(
        user_wrapper_token_account.lamports() > 0,
        TokenWrapperError::ExpectedInitializedAccount,
        "The account is not initialized, expected to be initialized",
    )?;

    let user_token_2022_token_account_data = user_token_2022_token_account.try_borrow_data()?;
    let (user_token_2022_token_account_data_stripped, _) = user_token_2022_token_account_data.split_at(spl_token::state::Account::LEN);
    let user_token_2022_data = spl_token_2022::state::Account::unpack(&user_token_2022_token_account_data_stripped).unwrap();
    drop(user_token_2022_token_account_data);
    assert_with_msg(
        &user_token_2022_data.owner == user_authority.key,
        TokenWrapperError::UnexpectedUserTokenAccountOwner,
        "User does not own the token account for this Token 2022 token",
    )?;

    let user_wrapper_token_account_data = user_wrapper_token_account.try_borrow_data()?;
    let (user_wrapper_token_account_data_stripped, _) = user_wrapper_token_account_data.split_at(spl_token::state::Account::LEN);
    let user_wrapper_token_data = spl_token::state::Account::unpack(&user_wrapper_token_account_data_stripped).unwrap();
    drop(user_wrapper_token_account_data);
    assert_with_msg(
        &user_wrapper_token_data.owner == user_authority.key,
        TokenWrapperError::UnexpectedUserTokenAccountOwner,
        "User does not own the token account for the wrapper token",
    )?;

    let reserve_token_2022_token_account_data = reserve_token_2022_token_account.try_borrow_data()?;
    let (reserve_token_2022_token_account_data_stripped, _) = reserve_token_2022_token_account_data.split_at(spl_token::state::Account::LEN);
    let reserve_token_2022_data = spl_token_2022::state::Account::unpack(&reserve_token_2022_token_account_data_stripped).unwrap();
    drop(reserve_token_2022_token_account_data);
    assert_with_msg(
        &reserve_token_2022_data.owner == reserve_authority.key,
        TokenWrapperError::UnexpectedReserveTokenAccountOwner,
        "The reserve does not own the token account for this Token 2022 token",
    )?;

    let user_burn_ix = spl_token::instruction::burn_checked(
        token_program.key,
        user_wrapper_token_account.key,
        wrapper_token_mint.key,
        user_authority.key,
        &[user_authority.key],
        amount,
        token_2022_mint_data.decimals,
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
        token_2022_mint_data.decimals,
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

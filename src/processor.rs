use solana_program::{
    account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey
};

use crate::{
    error::TokenWrapperError,
    instruction::TokenWrapperInstruction,
    utils::{
        assert_associated_token_program, assert_is_account_initialized,
        assert_is_account_uninitialized, assert_mint_authority, assert_rent,
        assert_reserve_authority, assert_system_program, assert_token_2022_program,
        assert_token_program, assert_vanilla_token_mint, assert_with_msg, get_reserve_authority,
        get_token_freeze_authority, get_token_mint_authority, get_vanilla_token_mint,
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
        TokenWrapperInstruction::InitializeToken => process_initialize_token(program_id, accounts),
        TokenWrapperInstruction::DepositAndMintTokens => {
            let (amount, _) = TokenWrapperInstruction::unpack_u64(data)?;

            process_deposit_and_mint_tokens(program_id, accounts, amount)
        }
        TokenWrapperInstruction::WithdrawAndBurnTokens => {
            let (amount, _) = TokenWrapperInstruction::unpack_u64(data)?;

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
    let reserve_authority = next_account_info(accounts_info_iter)?;
    let reserve_token_2022_token_account = next_account_info(accounts_info_iter)?;
    let token_program = next_account_info(accounts_info_iter)?;
    let system_program = next_account_info(accounts_info_iter)?;
    let rent_sysvar = next_account_info(accounts_info_iter)?;

    let (mint_authority, _, _) = get_token_mint_authority(*vanilla_token_mint.key, *program_id);
    let (freeze_authority, _, _) = get_token_freeze_authority(*vanilla_token_mint.key, *program_id);

    let unwrapped_token_2022_mint_data = token_2022_mint.try_borrow_data()?;
    let token_2022_mint_data =
        spl_token_2022::state::Mint::unpack(&unwrapped_token_2022_mint_data)?;

    assert_vanilla_token_mint(*token_2022_mint.key, *program_id, *vanilla_token_mint.key)?;
    assert_is_account_uninitialized(vanilla_token_mint)?;
    assert_is_account_uninitialized(reserve_authority)?;
    assert_is_account_uninitialized(reserve_token_2022_token_account)?;

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

    let (_, _, vanilla_token_mint_seeds) =
        get_vanilla_token_mint(*token_2022_mint.key, *program_id);

    invoke_signed(
        &init_mint_ix,
        &[
            vanilla_token_mint.clone(),
            rent_sysvar.clone(),
            token_program.clone(),
            payer.clone(),
        ],
        &[vanilla_token_mint_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    let (_, _, reserve_authority_seeds) = get_reserve_authority(*token_2022_mint.key, *program_id);

    let ata_init_ix = spl_associated_token_account::instruction::create_associated_token_account(
        payer.key,
        reserve_token_2022_token_account.key,
        token_2022_mint.key,
        token_program.key,
    );

    invoke_signed(
        &ata_init_ix,
        &[
            payer.clone(),
            reserve_token_2022_token_account.clone(),
            token_2022_mint.clone(),
            system_program.clone(),
            token_program.clone(),
        ],
        &[reserve_authority_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    Ok(())
}

pub fn process_deposit_and_mint_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("TokenWrapperInstruction::DepositAndMintTokens");

    let accounts_info_iter = &mut accounts.iter();
    let user_authority = next_account_info(accounts_info_iter)?;
    let reserve_authority = next_account_info(accounts_info_iter)?;
    let mint_authority = next_account_info(accounts_info_iter)?;
    let token_2022_mint = next_account_info(accounts_info_iter)?;
    let vanilla_token_mint = next_account_info(accounts_info_iter)?;
    let user_vanilla_token_account = next_account_info(accounts_info_iter)?;
    let user_token_2022_token_account = next_account_info(accounts_info_iter)?;
    let reserve_token_2022_token_account = next_account_info(accounts_info_iter)?;
    let token_program = next_account_info(accounts_info_iter)?;
    let token_2022_program = next_account_info(accounts_info_iter)?;
    let associated_token_program = next_account_info(accounts_info_iter)?;
    let system_program = next_account_info(accounts_info_iter)?;
    let rent_sysvar = next_account_info(accounts_info_iter)?;

    assert_vanilla_token_mint(*token_2022_mint.key, *program_id, *vanilla_token_mint.key)?;
    assert_is_account_initialized(vanilla_token_mint)?;
    assert_reserve_authority(*token_2022_mint.key, *program_id, *reserve_authority.key)?;
    assert_mint_authority(*vanilla_token_mint.key, *program_id, *mint_authority.key)?;

    assert_token_program(*token_program.key)?;
    assert_token_2022_program(*token_2022_program.key)?;
    assert_associated_token_program(*associated_token_program.key)?;
    assert_system_program(*system_program.key)?;
    assert_rent(*rent_sysvar.key)?;

    let unwrapped_token_2022_mint_data = token_2022_mint.try_borrow_data()?;
    let token_2022_mint_data =
        spl_token_2022::state::Mint::unpack(&unwrapped_token_2022_mint_data)?;

    if user_vanilla_token_account.lamports() == 0 {
        let ata_init_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                user_authority.key,
                user_vanilla_token_account.key,
                vanilla_token_mint.key,
                token_program.key,
            );

        invoke(
            &ata_init_ix,
            &[
                user_authority.clone(),
                user_vanilla_token_account.clone(),
                vanilla_token_mint.clone(),
                system_program.clone(),
                token_program.clone(),
            ],
        )?;
    }

    assert_with_msg(
        user_token_2022_token_account.owner == user_authority.key,
        TokenWrapperError::UnexpectedUserTokenAccountOwner,
        "User does not own the token account for this Token 2022 token",
    )?;
    assert_with_msg(
        user_vanilla_token_account.owner == user_authority.key,
        TokenWrapperError::UnexpectedUserTokenAccountOwner,
        "User does not own the token account for this Token 2022 token",
    )?;
    assert_with_msg(
        reserve_token_2022_token_account.owner == reserve_authority.key,
        TokenWrapperError::UnexpectedReserveTokenAccountOwner,
        "The reserve does not own the token account for this Token 2022 token",
    )?;

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

    let (_, _, mint_authority_seeds) =
        get_token_mint_authority(*vanilla_token_mint.key, *program_id);

    let user_mint_ix = spl_token::instruction::mint_to_checked(
        token_program.key,
        vanilla_token_mint.key,
        user_vanilla_token_account.key,
        user_authority.key,
        &[reserve_authority.key],
        amount,
        token_2022_mint_data.decimals,
    )?;

    invoke_signed(
        &user_mint_ix,
        &[
            token_program.clone(),
            user_vanilla_token_account.clone(),
            vanilla_token_mint.clone(),
            user_authority.clone(),
        ],
        &[mint_authority_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    Ok(())
}

pub fn process_withdraw_and_burn_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("TokenWrapperInstruction::WithdrawAndBurnTokens");

    let accounts_info_iter = &mut accounts.iter();
    let user_authority = next_account_info(accounts_info_iter)?;
    let reserve_authority = next_account_info(accounts_info_iter)?;
    let token_2022_mint = next_account_info(accounts_info_iter)?;
    let vanilla_token_mint = next_account_info(accounts_info_iter)?;
    let user_vanilla_token_account = next_account_info(accounts_info_iter)?;
    let user_token_2022_token_account = next_account_info(accounts_info_iter)?;
    let reserve_token_2022_token_account = next_account_info(accounts_info_iter)?;
    let token_program = next_account_info(accounts_info_iter)?;
    let token_2022_program = next_account_info(accounts_info_iter)?;
    let system_program = next_account_info(accounts_info_iter)?;
    let rent_sysvar = next_account_info(accounts_info_iter)?;

    // TODO: Validate if the token accounts are associated token accounts

    assert_vanilla_token_mint(*token_2022_mint.key, *program_id, *vanilla_token_mint.key)?;
    assert_is_account_initialized(vanilla_token_mint)?;
    assert_reserve_authority(*token_2022_mint.key, *program_id, *reserve_authority.key)?;

    assert_token_program(*token_program.key)?;
    assert_token_2022_program(*token_2022_program.key)?;
    assert_system_program(*system_program.key)?;
    assert_rent(*rent_sysvar.key)?;

    let unwrapped_token_2022_mint_data = token_2022_mint.try_borrow_data()?;
    let token_2022_mint_data =
        spl_token_2022::state::Mint::unpack(&unwrapped_token_2022_mint_data)?;

    assert_with_msg(
        user_vanilla_token_account.lamports() > 0,
        TokenWrapperError::ExpectedInitializedAccount,
        "The account is not initialized, expected to be initialized",
    )?;
    assert_with_msg(
        user_token_2022_token_account.owner == user_authority.key,
        TokenWrapperError::UnexpectedUserTokenAccountOwner,
        "User does not own the token account for this Token 2022 token",
    )?;
    assert_with_msg(
        user_vanilla_token_account.owner == user_authority.key,
        TokenWrapperError::UnexpectedUserTokenAccountOwner,
        "User does not own the token account for this Token 2022 token",
    )?;
    assert_with_msg(
        reserve_token_2022_token_account.owner == reserve_authority.key,
        TokenWrapperError::UnexpectedReserveTokenAccountOwner,
        "The reserve does not own the token account for this Token 2022 token",
    )?;

    let user_burn_ix = spl_token::instruction::burn_checked(
        token_program.key,
        user_vanilla_token_account.key,
        vanilla_token_mint.key,
        user_authority.key,
        &[user_authority.key],
        amount,
        token_2022_mint_data.decimals,
    )?;

    invoke(
        &user_burn_ix,
        &[
            token_program.clone(),
            user_vanilla_token_account.clone(),
            vanilla_token_mint.clone(),
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
            user_token_2022_token_account.clone(),
            token_2022_mint.clone(),
            reserve_token_2022_token_account.clone(),
            user_authority.clone(),
        ],
        &[reserve_authority_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    Ok(())
}

pub mod utils;

use std::time::Duration;

use crate::utils::TestClient;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey, signature::Keypair, signer::Signer};
use spl_associated_token_account::get_associated_token_address;
use token2022_wrapper::{
    instruction_builders::{
        create_deposit_and_mint_wrapper_tokens_instruction,
        create_initialize_wrapper_token_instruction,
        create_withdraw_and_burn_wrapper_tokens_instruction,
    },
    utils::get_wrapper_token_mint,
};
use utils::{
    airdrop, assert_with_msg, create_associated_token_account, create_mint, create_token_2022_mint,
    create_token_account_token_2022, extract_error_code, get_token_balance, get_token_balance_2022,
    get_token_mint, mint_token_2022_tokens, mint_tokens, sign_send_instructions,
    test_transfer_fee_config_with_keypairs, TransferFeeConfigWithKeypairs,
};

pub const PROGRAM_ID: Pubkey = pubkey!("6E9iP7p4Gx2e6c2Yt4MHY5T1aZ8RWhrmF9p6bXkGWiza");

pub async fn create_and_mint_tokens(
    client: &mut TestClient,
    recipient: &Pubkey,
    amount: u64,
    decimals: u8,
) -> (Pubkey, Pubkey) {
    let payer_keypair = client.get_payer_clone();

    let token_mint = create_mint(client, &payer_keypair.pubkey(), None, decimals, None)
        .await
        .unwrap();

    let associated_token_account =
        create_associated_token_account(client, recipient, &token_mint, &spl_token::id())
            .await
            .unwrap();

    let _ = mint_tokens(
        client,
        &payer_keypair,
        &token_mint,
        &associated_token_account,
        amount,
        None,
    )
    .await;

    (token_mint, associated_token_account)
}

pub async fn create_and_mint_tokens_token_2022(
    client: &mut TestClient,
    recipient: &Pubkey,
    amount: u64,
    decimals: u8,
    transfer_fee_config: Option<&TransferFeeConfigWithKeypairs>,
) -> (Pubkey, Pubkey) {
    let payer_keypair = client.get_payer_clone();

    let token_mint = create_token_2022_mint(
        client,
        &payer_keypair.pubkey(),
        Some(&payer_keypair.pubkey()),
        decimals,
        None,
        transfer_fee_config,
    )
    .await
    .unwrap();

    let token_account = create_token_account_token_2022(client, &recipient, &token_mint)
        .await
        .unwrap();

    let _ = mint_token_2022_tokens(
        client,
        &payer_keypair,
        &token_mint,
        &token_account,
        amount,
        None,
    )
    .await;

    (token_mint, token_account)
}

pub async fn create_and_mint_frozen_tokens_token_2022(
    client: &mut TestClient,
    recipient: &Pubkey,
    amount: u64,
    decimals: u8,
) -> (Pubkey, Pubkey, bool) {
    let payer_keypair = client.get_payer_clone();

    let token_mint = create_token_2022_mint(
        client,
        &payer_keypair.pubkey(),
        Some(&payer_keypair.pubkey()),
        decimals,
        None,
        None,
    )
    .await
    .unwrap();

    let token_account = create_token_account_token_2022(client, &recipient, &token_mint)
        .await
        .unwrap();

    let _ = mint_token_2022_tokens(
        client,
        &payer_keypair,
        &token_mint,
        &token_account,
        amount,
        None,
    )
    .await;

    let freeze_ix = spl_token_2022::instruction::freeze_account(
        &spl_token_2022::id(),
        &token_account,
        &token_mint,
        &payer_keypair.pubkey(),
        &[&payer_keypair.pubkey()],
    )
    .unwrap();

    let status =
        match sign_send_instructions(client, &[freeze_ix], vec![&payer_keypair], None).await {
            Ok(_) => true,
            Err(_) => false,
        };

    (token_mint, token_account, status)
}

mod tests {

    use token2022_wrapper::{error::TokenWrapperError, utils::get_reserve_authority};

    use super::*;

    /// Test 1 - testing successful initialization of a wrapper token mint for a Token 2022 mint
    ///
    ///
    #[tokio::test]
    async fn test_1() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 9_u8;
        let amount_2022 = 10_000_u64 * 10_u64.pow(decimal_2022 as u32);

        let (token_2022_mint, _) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let (wrapper_token, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);
                let (expected_mint_authority, _, _) =
                    get_reserve_authority(token_2022_mint, PROGRAM_ID);
                let (expected_freeze_authority, _, _) =
                    get_reserve_authority(token_2022_mint, PROGRAM_ID);

                let wrapper_token_ac = get_token_mint(&mut test_client, &wrapper_token)
                    .await
                    .unwrap();

                assert_with_msg(
                    expected_mint_authority == wrapper_token_ac.mint_authority.unwrap(),
                    "Mint authority for the wrapper token does not match",
                );
                assert_with_msg(
                    expected_freeze_authority == wrapper_token_ac.freeze_authority.unwrap(),
                    "Freeze authority for the wrapper token does not match",
                );
                assert_with_msg(
                    wrapper_token_ac.decimals == decimal_2022,
                    "Decimals for the wrapper token does not match",
                );
                assert_with_msg(
                    wrapper_token_ac.supply == 0,
                    "Invalid initial supply for the wrapper token",
                );
            }
            Err(e) => {
                println!("Error: {}", e);
                panic!("test_1 error: {}", e);
            }
        };
    }

    /// Test 2 - cannot initialize on repeated tokens
    ///
    ///
    #[tokio::test]
    async fn test_2() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 9_u8;
        let amount_2022 = 10_000_u64 * 10_u64.pow(decimal_2022 as u32);

        let (token_2022_mint, _) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let duplicate_initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await;

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![duplicate_initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                panic!("Expected test_2 to fail, but succeeded");
            }
            Err(e) => {
                let _ = match extract_error_code(e.to_string().as_str()) {
                    Some(error_code) => {
                        assert_with_msg(
                            error_code == TokenWrapperError::UnexpectedWrapperToken as u32,
                            format!("Invalid error thrown for test_2: {}", e).as_str(),
                        );
                    }
                    None => {
                        println!("Could not parse error code from the BanksClientError");
                        panic!("Could not parse error code from the BanksClientError");
                    }
                };
            }
        };
    }

    /// Test 3 - cannot initialize for an spl token
    ///
    ///
    #[tokio::test]
    async fn test_3() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal = 9_u8;
        let amount = 10_000_u64 * 10_u64.pow(decimal as u32);

        let (token_mint, _) =
            create_and_mint_tokens(&mut test_client, &user.pubkey(), amount, decimal).await;

        let token_data = get_token_mint(&mut test_client, &token_mint).await.unwrap();

        assert_with_msg(token_data.decimals == decimal, "Invalid token decimals");

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                panic!("Expected test_3 to fail, but succeeded");
            }
            Err(e) => {
                assert_with_msg(
                    e.to_string().contains("incorrect program id"),
                    "Expected test_3 to fail with incorrect program id",
                );
            }
        };
    }

    /// Test 4 - mint test tokens with decimal 5
    ///
    ///
    #[tokio::test]
    async fn test_4() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 5_u8;
        let amount_2022 = 10_000_u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;
        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let user_token_2022_before_balance =
            get_token_balance(&mut test_client, &user_token_2022_token_account).await;
        let user_wrapper_before_balance =
            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        tokio::time::sleep(Duration::from_millis(1_000)).await;

                        let user_token_2022_after_balance =
                            get_token_balance(&mut test_client, &user_token_2022_token_account)
                                .await;
                        let user_wrapper_after_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        assert_with_msg(
                            user_token_2022_after_balance
                                == user_token_2022_before_balance - amount_wrapper,
                            "Invalid user Token2022 token balance change",
                        );
                        assert_with_msg(
                            (user_wrapper_after_balance
                                == user_wrapper_before_balance + amount_wrapper)
                                && (user_wrapper_after_balance == amount_wrapper),
                            "Invalid user wrapper token balance change",
                        );
                    }
                    Err(e) => {
                        println!("Error deposit tx: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_4 error: {}", e);
            }
        };
    }

    /// Test 5 - mint test tokens with decimal 8
    ///
    ///
    #[tokio::test]
    async fn test_5() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 8_u8;
        let amount_2022 = 10_000_u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;
        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let user_token_2022_before_balance =
            get_token_balance(&mut test_client, &user_token_2022_token_account).await;
        let user_wrapper_before_balance =
            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        tokio::time::sleep(Duration::from_millis(1_000)).await;

                        let user_token_2022_after_balance =
                            get_token_balance(&mut test_client, &user_token_2022_token_account)
                                .await;
                        let user_wrapper_after_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        assert_with_msg(
                            user_token_2022_after_balance
                                == user_token_2022_before_balance - amount_wrapper,
                            "Invalid user Token2022 token balance change",
                        );
                        assert_with_msg(
                            (user_wrapper_after_balance
                                == user_wrapper_before_balance + amount_wrapper)
                                && (user_wrapper_after_balance == amount_wrapper),
                            "Invalid user wrapper token balance change",
                        );
                    }
                    Err(e) => {
                        println!("Error deposit tx: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_5 error: {}", e);
            }
        };
    }

    /// Test 6 - mint test tokens with decimal 1
    ///
    ///
    #[tokio::test]
    async fn test_6() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 1_u8;
        let amount_2022 = 10_000_u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;
        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let user_token_2022_before_balance =
            get_token_balance(&mut test_client, &user_token_2022_token_account).await;
        let user_wrapper_before_balance =
            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        tokio::time::sleep(Duration::from_millis(1_000)).await;

                        let user_token_2022_after_balance =
                            get_token_balance(&mut test_client, &user_token_2022_token_account)
                                .await;
                        let user_wrapper_after_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        assert_with_msg(
                            user_token_2022_after_balance
                                == user_token_2022_before_balance - amount_wrapper,
                            "Invalid user Token2022 token balance change",
                        );
                        assert_with_msg(
                            (user_wrapper_after_balance
                                == user_wrapper_before_balance + amount_wrapper)
                                && (user_wrapper_after_balance == amount_wrapper),
                            "Invalid user wrapper token balance change",
                        );
                    }
                    Err(e) => {
                        println!("Error deposit tx: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_6 error: {}", e);
            }
        };
    }

    /// Test 7 - mint test tokens with decimal 0
    ///
    ///
    #[tokio::test]
    async fn test_7() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 0_u8;
        let amount_2022 = 10_000_u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;
        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let user_token_2022_before_balance =
            get_token_balance(&mut test_client, &user_token_2022_token_account).await;
        let user_wrapper_before_balance =
            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        tokio::time::sleep(Duration::from_millis(1_000)).await;

                        let user_token_2022_after_balance =
                            get_token_balance(&mut test_client, &user_token_2022_token_account)
                                .await;
                        let user_wrapper_after_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        assert_with_msg(
                            user_token_2022_after_balance
                                == user_token_2022_before_balance - amount_wrapper,
                            "Invalid user Token2022 token balance change",
                        );
                        assert_with_msg(
                            (user_wrapper_after_balance
                                == user_wrapper_before_balance + amount_wrapper)
                                && (user_wrapper_after_balance == amount_wrapper),
                            "Invalid user wrapper token balance change",
                        );
                    }
                    Err(e) => {
                        println!("Error deposit tx: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_7 error: {}", e);
            }
        };
    }

    /// Test 8 - cannot mint if not owned
    ///
    ///
    #[tokio::test]
    async fn test_8() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 5_u8;
        let amount_2022 = 0u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = 2u64;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;
        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        panic!("Expected test_8 to fail, but succeeded");
                    }
                    Err(e) => {
                        let _ = match extract_error_code(e.to_string().as_str()) {
                            Some(error_code) => {
                                assert_with_msg(
                                    error_code == 1 as u32, // Error code 0x1 --> Insufficient funds
                                    format!("Invalid error thrown for test_8: {}", e).as_str(),
                                );
                            }
                            None => {
                                println!("Could not parse error code from the BanksClientError");
                                panic!("Could not parse error code from the BanksClientError");
                            }
                        };
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_8 error: {}", e);
            }
        };
    }

    /// Test 9 - cannot mint with an invalid token deposit
    ///
    ///
    #[tokio::test]
    async fn test_9() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 5_u8;
        let amount_2022 = 10_000u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;

        let (token_2022_mint, _) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;

        let (token_2022_mint_secondary, user_token_2022_token_account_secondary) =
            create_and_mint_tokens_token_2022(
                &mut test_client,
                &user.pubkey(),
                amount_2022,
                decimal_2022,
                None,
            )
            .await;

        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint_secondary,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account_secondary,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        panic!("Expected test_9 to fail, but succeeded");
                    }
                    Err(e) => {
                        let _ = match extract_error_code(e.to_string().as_str()) {
                            Some(error_code) => {
                                assert_with_msg(
                                    error_code == TokenWrapperError::UnexpectedWrapperToken as u32,
                                    format!("Invalid error thrown for test_9: {}", e).as_str(),
                                );
                            }
                            None => {
                                println!("Could not parse error code from the BanksClientError");
                                panic!("Could not parse error code from the BanksClientError");
                            }
                        };
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_9 error: {}", e);
            }
        };
    }

    /// Test 10 - cannot mint if tokens are frozen
    ///
    ///
    #[tokio::test]
    async fn test_10() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 5_u8;
        let amount_2022 = 0u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = 2u64;

        let (token_2022_mint, user_token_2022_token_account, _) =
            create_and_mint_frozen_tokens_token_2022(
                &mut test_client,
                &user.pubkey(),
                amount_2022,
                decimal_2022,
            )
            .await;

        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        panic!("Expected test_10 to fail, but succeeded");
                    }
                    Err(e) => {
                        let _ = match extract_error_code(e.to_string().as_str()) {
                            Some(error_code) => {
                                assert_with_msg(
                                    error_code == 17 as u32, // Error code 17 --> Account is frozen
                                    format!("Invalid error thrown for test_10: {}", e).as_str(),
                                );
                            }
                            None => {
                                println!("Could not parse error code from the BanksClientError");
                                panic!("Could not parse error code from the BanksClientError");
                            }
                        };
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_10 error: {}", e);
            }
        };
    }

    /// Test 11 - burn test tokens with decimal 5
    ///
    ///
    #[tokio::test]
    async fn test_11() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 5_u8;
        let amount_2022 = 10_000u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;

        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let user_token_2022_before_balance =
            get_token_balance(&mut test_client, &user_token_2022_token_account).await;
        let user_wrapper_before_balance =
            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        let user_token_2022_after_balance =
                            get_token_balance(&mut test_client, &user_token_2022_token_account)
                                .await;
                        let user_wrapper_after_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        assert_with_msg(
                            user_token_2022_after_balance
                                == user_token_2022_before_balance - amount_wrapper,
                            "Invalid user Token2022 token balance change",
                        );
                        assert_with_msg(
                            (user_wrapper_after_balance
                                == user_wrapper_before_balance + amount_wrapper)
                                && (user_wrapper_after_balance == amount_wrapper),
                            "Invalid user wrapper token balance change",
                        );

                        let burn_ix = create_withdraw_and_burn_wrapper_tokens_instruction(
                            &user.pubkey(),
                            &token_2022_mint,
                            &user_wrapper_token_account,
                            &user_token_2022_token_account,
                            amount_wrapper,
                        );

                        let _ = match sign_send_instructions(
                            &mut test_client,
                            &vec![burn_ix],
                            vec![&user, &payer_keypair],
                            None,
                        )
                        .await
                        {
                            Ok(_) => {
                                let user_token_2022_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_token_2022_token_account,
                                )
                                .await;
                                let user_wrapper_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_wrapper_token_account,
                                )
                                .await;

                                assert_with_msg(
                                    user_token_2022_before_balance
                                        == user_token_2022_after_burn_balance,
                                    "Invalid user Token2022 token after burn balance change",
                                );
                                assert_with_msg(
                                    user_wrapper_after_burn_balance == 0,
                                    "Invalid user wrapper token after burn balance change",
                                );
                            }
                            Err(e) => {
                                println!("Error burning wrapper tokens transaction: {}", e);
                            }
                        };
                    }
                    Err(e) => {
                        println!("Error minting wrapper tokens: {}", e);
                        panic!("test_11 error: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_11 error: {}", e);
            }
        };
    }

    /// Test 12 - burn test tokens with decimal 8
    ///
    ///
    #[tokio::test]
    async fn test_12() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 8_u8;
        let amount_2022 = 10_000u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;

        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let user_token_2022_before_balance =
            get_token_balance(&mut test_client, &user_token_2022_token_account).await;
        let user_wrapper_before_balance =
            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        let user_token_2022_after_balance =
                            get_token_balance(&mut test_client, &user_token_2022_token_account)
                                .await;
                        let user_wrapper_after_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        assert_with_msg(
                            user_token_2022_after_balance
                                == user_token_2022_before_balance - amount_wrapper,
                            "Invalid user Token2022 token balance change",
                        );
                        assert_with_msg(
                            (user_wrapper_after_balance
                                == user_wrapper_before_balance + amount_wrapper)
                                && (user_wrapper_after_balance == amount_wrapper),
                            "Invalid user wrapper token balance change",
                        );

                        let burn_ix = create_withdraw_and_burn_wrapper_tokens_instruction(
                            &user.pubkey(),
                            &token_2022_mint,
                            &user_wrapper_token_account,
                            &user_token_2022_token_account,
                            amount_wrapper,
                        );

                        let _ = match sign_send_instructions(
                            &mut test_client,
                            &vec![burn_ix],
                            vec![&user, &payer_keypair],
                            None,
                        )
                        .await
                        {
                            Ok(_) => {
                                let user_token_2022_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_token_2022_token_account,
                                )
                                .await;
                                let user_wrapper_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_wrapper_token_account,
                                )
                                .await;

                                assert_with_msg(
                                    user_token_2022_before_balance
                                        == user_token_2022_after_burn_balance,
                                    "Invalid user Token2022 token after burn balance change",
                                );
                                assert_with_msg(
                                    user_wrapper_after_burn_balance == 0,
                                    "Invalid user wrapper token after burn balance change",
                                );
                            }
                            Err(e) => {
                                println!("Error burning wrapper tokens transaction: {}", e);
                            }
                        };
                    }
                    Err(e) => {
                        println!("Error minting wrapper tokens: {}", e);
                        panic!("test_12 error: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_12 error: {}", e);
            }
        };
    }

    /// Test 13 - burn test tokens with decimal 1
    ///
    ///
    #[tokio::test]
    async fn test_13() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 1_u8;
        let amount_2022 = 10_000u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;

        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let user_token_2022_before_balance =
            get_token_balance(&mut test_client, &user_token_2022_token_account).await;
        let user_wrapper_before_balance =
            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        let user_token_2022_after_balance =
                            get_token_balance(&mut test_client, &user_token_2022_token_account)
                                .await;
                        let user_wrapper_after_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        assert_with_msg(
                            user_token_2022_after_balance
                                == user_token_2022_before_balance - amount_wrapper,
                            "Invalid user Token2022 token balance change",
                        );
                        assert_with_msg(
                            (user_wrapper_after_balance
                                == user_wrapper_before_balance + amount_wrapper)
                                && (user_wrapper_after_balance == amount_wrapper),
                            "Invalid user wrapper token balance change",
                        );

                        let burn_ix = create_withdraw_and_burn_wrapper_tokens_instruction(
                            &user.pubkey(),
                            &token_2022_mint,
                            &user_wrapper_token_account,
                            &user_token_2022_token_account,
                            amount_wrapper,
                        );

                        let _ = match sign_send_instructions(
                            &mut test_client,
                            &vec![burn_ix],
                            vec![&user, &payer_keypair],
                            None,
                        )
                        .await
                        {
                            Ok(_) => {
                                let user_token_2022_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_token_2022_token_account,
                                )
                                .await;
                                let user_wrapper_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_wrapper_token_account,
                                )
                                .await;

                                assert_with_msg(
                                    user_token_2022_before_balance
                                        == user_token_2022_after_burn_balance,
                                    "Invalid user Token2022 token after burn balance change",
                                );
                                assert_with_msg(
                                    user_wrapper_after_burn_balance == 0,
                                    "Invalid user wrapper token after burn balance change",
                                );
                            }
                            Err(e) => {
                                println!("Error burning wrapper tokens transaction: {}", e);
                            }
                        };
                    }
                    Err(e) => {
                        println!("Error minting wrapper tokens: {}", e);
                        panic!("test_13 error: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_13 error: {}", e);
            }
        };
    }

    /// Test 14 - burn test tokens with decimal 0
    ///
    ///
    #[tokio::test]
    async fn test_14() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 0_u8;
        let amount_2022 = 10_000u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;

        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let user_token_2022_before_balance =
            get_token_balance(&mut test_client, &user_token_2022_token_account).await;
        let user_wrapper_before_balance =
            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        let user_token_2022_after_balance =
                            get_token_balance(&mut test_client, &user_token_2022_token_account)
                                .await;
                        let user_wrapper_after_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        assert_with_msg(
                            user_token_2022_after_balance
                                == user_token_2022_before_balance - amount_wrapper,
                            "Invalid user Token2022 token balance change",
                        );
                        assert_with_msg(
                            (user_wrapper_after_balance
                                == user_wrapper_before_balance + amount_wrapper)
                                && (user_wrapper_after_balance == amount_wrapper),
                            "Invalid user wrapper token balance change",
                        );

                        let burn_ix = create_withdraw_and_burn_wrapper_tokens_instruction(
                            &user.pubkey(),
                            &token_2022_mint,
                            &user_wrapper_token_account,
                            &user_token_2022_token_account,
                            amount_wrapper,
                        );

                        let _ = match sign_send_instructions(
                            &mut test_client,
                            &vec![burn_ix],
                            vec![&user, &payer_keypair],
                            None,
                        )
                        .await
                        {
                            Ok(_) => {
                                let user_token_2022_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_token_2022_token_account,
                                )
                                .await;
                                let user_wrapper_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_wrapper_token_account,
                                )
                                .await;

                                assert_with_msg(
                                    user_token_2022_before_balance
                                        == user_token_2022_after_burn_balance,
                                    "Invalid user Token2022 token after burn balance change",
                                );
                                assert_with_msg(
                                    user_wrapper_after_burn_balance == 0,
                                    "Invalid user wrapper token after burn balance change",
                                );
                            }
                            Err(e) => {
                                println!("Error burning wrapper tokens transaction: {}", e);
                            }
                        };
                    }
                    Err(e) => {
                        println!("Error minting wrapper tokens: {}", e);
                        panic!("test_14 error: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_14 error: {}", e);
            }
        };
    }

    /// Test 15 - cannot burn if not owned
    ///
    ///
    #[tokio::test]
    async fn test_15() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 0_u8;
        let amount_2022 = 10_000u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;
        let amount_wrapper_burn = amount_wrapper + 1;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;

        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let user_token_2022_before_balance =
            get_token_balance(&mut test_client, &user_token_2022_token_account).await;
        let user_wrapper_before_balance =
            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        let user_token_2022_after_balance =
                            get_token_balance(&mut test_client, &user_token_2022_token_account)
                                .await;
                        let user_wrapper_after_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        assert_with_msg(
                            user_token_2022_after_balance
                                == user_token_2022_before_balance - amount_wrapper,
                            "Invalid user Token2022 token balance change",
                        );
                        assert_with_msg(
                            (user_wrapper_after_balance
                                == user_wrapper_before_balance + amount_wrapper)
                                && (user_wrapper_after_balance == amount_wrapper),
                            "Invalid user wrapper token balance change",
                        );

                        let burn_ix = create_withdraw_and_burn_wrapper_tokens_instruction(
                            &user.pubkey(),
                            &token_2022_mint,
                            &user_wrapper_token_account,
                            &user_token_2022_token_account,
                            amount_wrapper_burn,
                        );

                        let _ = match sign_send_instructions(
                            &mut test_client,
                            &vec![burn_ix],
                            vec![&user, &payer_keypair],
                            None,
                        )
                        .await
                        {
                            Ok(_) => {
                                panic!("Expected test_15 to fail, but succeeded");
                            }
                            Err(e) => {
                                let _ = match extract_error_code(e.to_string().as_str()) {
                                    Some(error_code) => {
                                        let user_token_2022_after_burn_balance = get_token_balance(
                                            &mut test_client,
                                            &user_token_2022_token_account,
                                        )
                                        .await;
                                        let user_wrapper_after_burn_balance = get_token_balance(
                                            &mut test_client,
                                            &user_wrapper_token_account,
                                        )
                                        .await;

                                        assert_with_msg(
                                        user_token_2022_after_balance
                                            == user_token_2022_after_burn_balance,
                                        "Invalid user Token2022 token after burn balance change",
                                    );
                                        assert_with_msg(
                                            user_wrapper_after_balance
                                                == user_wrapper_after_burn_balance,
                                            "Invalid user wrapper token after burn balance change",
                                        );

                                        assert_with_msg(
                                            error_code == 1 as u32, // Error code 0x1 --> Insufficient funds
                                            format!("Invalid error thrown for test_15: {}", e)
                                                .as_str(),
                                        );
                                    }
                                    None => {
                                        println!(
                                            "Could not parse error code from the BanksClientError"
                                        );
                                        panic!("Could not parse error code from the BanksClientError");
                                    }
                                };
                            }
                        };
                    }
                    Err(e) => {
                        println!("Error minting wrapper tokens: {}", e);
                        panic!("test_15 error: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_15 error: {}", e);
            }
        };
    }

    /// Test 16 - cannot burn for an invalid token22 deposit
    ///
    ///
    #[tokio::test]
    async fn test_16() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 0_u8;
        let amount_2022 = 10_000u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;
        let amount_wrapper_burn = amount_wrapper + 1;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;

        let (token_2022_mint_duplicate, user_token_2022_token_account_duplicate) =
            create_and_mint_tokens_token_2022(
                &mut test_client,
                &user.pubkey(),
                amount_2022,
                decimal_2022,
                None,
            )
            .await;

        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let user_token_2022_before_balance =
            get_token_balance(&mut test_client, &user_token_2022_token_account).await;
        let user_wrapper_before_balance =
            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        let user_token_2022_after_balance =
                            get_token_balance(&mut test_client, &user_token_2022_token_account)
                                .await;
                        let user_wrapper_after_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        assert_with_msg(
                            user_token_2022_after_balance
                                == user_token_2022_before_balance - amount_wrapper,
                            "Invalid user Token2022 token balance change",
                        );
                        assert_with_msg(
                            (user_wrapper_after_balance
                                == user_wrapper_before_balance + amount_wrapper)
                                && (user_wrapper_after_balance == amount_wrapper),
                            "Invalid user wrapper token balance change",
                        );

                        let burn_ix = create_withdraw_and_burn_wrapper_tokens_instruction(
                            &user.pubkey(),
                            &token_2022_mint_duplicate,
                            &user_wrapper_token_account,
                            &user_token_2022_token_account_duplicate,
                            amount_wrapper_burn,
                        );

                        let _ = match sign_send_instructions(
                            &mut test_client,
                            &vec![burn_ix],
                            vec![&user, &payer_keypair],
                            None,
                        )
                        .await
                        {
                            Ok(_) => {
                                panic!("Expected test_16 to fail, but succeeded");
                            }
                            Err(e) => {
                                let _ = match extract_error_code(e.to_string().as_str()) {
                                    Some(error_code) => {
                                        assert_with_msg(
                                            error_code == TokenWrapperError::UnexpectedWrapperToken as u32,
                                            format!("Invalid error thrown for test_16: {}", e).as_str(),
                                        );
                                    }
                                    None => {
                                        println!("Could not parse error code from the BanksClientError");
                                        panic!("Could not parse error code from the BanksClientError");
                                    }
                                };
                            }
                        };
                    }
                    Err(e) => {
                        println!("Error minting wrapper tokens: {}", e);
                        panic!("test_16 error: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_16 error: {}", e);
            }
        };
    }

    /// Test 17 - works if user A mints, sends it to user B and user B withdraws
    ///
    ///
    #[tokio::test]
    async fn test_17() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let user_2 = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;
        let _ = airdrop(&mut test_client, &user_2.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 8_u8;
        let amount_2022 = 10_000u64 * 10_u64.pow(decimal_2022 as u32);
        let amount_wrapper = amount_2022 / 2;
        let amount_wrapper_burn = 10_u64;

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            None,
        )
        .await;

        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        let user_2_wrapper_token_account = create_associated_token_account(
                            &mut test_client,
                            &user_2.pubkey(),
                            &wrapper_token_mint,
                            &spl_token::id(),
                        )
                        .await
                        .unwrap();

                        let transfer_ix = spl_token::instruction::transfer_checked(
                            &spl_token::id(),
                            &user_wrapper_token_account,
                            &wrapper_token_mint,
                            &user_2_wrapper_token_account,
                            &user.pubkey(),
                            &[&user.pubkey()],
                            amount_wrapper_burn,
                            decimal_2022,
                        )
                        .unwrap();

                        let _ = sign_send_instructions(
                            &mut test_client,
                            &[transfer_ix],
                            vec![&user, &payer_keypair],
                            None,
                        )
                        .await;

                        let user_2_token_2022_token_account = create_token_account_token_2022(
                            &mut test_client,
                            &user_2.pubkey(),
                            &token_2022_mint,
                        )
                        .await
                        .unwrap();

                        let burn_ix = create_withdraw_and_burn_wrapper_tokens_instruction(
                            &user_2.pubkey(),
                            &token_2022_mint,
                            &user_2_wrapper_token_account,
                            &user_2_token_2022_token_account,
                            amount_wrapper_burn,
                        );

                        let _ = match sign_send_instructions(
                            &mut test_client,
                            &vec![burn_ix],
                            vec![&user_2, &payer_keypair],
                            None,
                        )
                        .await
                        {
                            Ok(_) => {
                                let user_2_token_2022_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_2_token_2022_token_account,
                                )
                                .await;
                                let user_2_wrapper_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_2_wrapper_token_account,
                                )
                                .await;

                                assert_with_msg(
                                    user_2_token_2022_after_burn_balance == amount_wrapper_burn,
                                    "Invalid user 2 Token2022 token after burn balance change",
                                );
                                assert_with_msg(
                                    user_2_wrapper_after_burn_balance == 0,
                                    "Invalid user 2 wrapper token after burn balance change",
                                );
                            }
                            Err(e) => {
                                println!("Error burn tx: {}", e);
                            }
                        };
                    }
                    Err(e) => {
                        println!("Error minting wrapper tokens: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_17 error: {}", e);
            }
        };
    }

    /// Test 18 - Transfer fee enabled 100bps (1%) - works if user A mints, sends it to user B and user B withdraws
    ///
    /// user A deposits 100 tokens
    /// reserve gets 99 tokens
    /// user A is minted 99 wTokens
    ///
    ///
    /// user A transfers 99 wTokens to user B
    ///
    ///
    /// user B burns 99 wTokens
    /// reserve sends 99 tokens
    /// user B receives 98.01 tokens
    ///
    ///
    /// Post flow balances:
    /// user A - token: 0, wToken: 0
    /// user B - token: 98.01, wToken: 0
    ///
    #[tokio::test]
    async fn test_18() {
        let mut test_client = TestClient::new().await;
        let payer_keypair = test_client.get_payer_clone();

        let user = Keypair::new();
        let user_2 = Keypair::new();
        let _ = airdrop(&mut test_client, &user.pubkey(), 5 * LAMPORTS_PER_SOL).await;
        let _ = airdrop(&mut test_client, &user_2.pubkey(), 5 * LAMPORTS_PER_SOL).await;

        let decimal_2022 = 8_u8;
        let amount_2022 = 10_000u64;
        let amount_wrapper = 100u64;
        let amount_wrapper_burn = 99u64;

        let transfer_fee_test_config = test_transfer_fee_config_with_keypairs();

        let (token_2022_mint, user_token_2022_token_account) = create_and_mint_tokens_token_2022(
            &mut test_client,
            &user.pubkey(),
            amount_2022,
            decimal_2022,
            Some(&transfer_fee_test_config), // None
        )
        .await;

        let (wrapper_token_mint, _, _) = get_wrapper_token_mint(token_2022_mint, PROGRAM_ID);

        let user_wrapper_token_account =
            get_associated_token_address(&user.pubkey(), &wrapper_token_mint);

        let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
            .await
            .unwrap();

        assert_with_msg(
            token_2022_data.decimals == decimal_2022,
            "Invalid token_2022 decimals",
        );

        let initialize_ix =
            create_initialize_wrapper_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

        let _ = match sign_send_instructions(
            &mut test_client,
            &vec![initialize_ix],
            vec![&payer_keypair],
            None,
        )
        .await
        {
            Ok(_sig) => {
                let deposit_ix = create_deposit_and_mint_wrapper_tokens_instruction(
                    &user.pubkey(),
                    &token_2022_mint,
                    &user_wrapper_token_account,
                    &user_token_2022_token_account,
                    amount_wrapper,
                );

                let _ = match sign_send_instructions(
                    &mut test_client,
                    &vec![deposit_ix],
                    vec![&user, &payer_keypair],
                    None,
                )
                .await
                {
                    Ok(_sig) => {
                        let user_wrapper_token_after_deposit_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        let user_token_2022_after_deposit_balance = get_token_balance_2022(
                            &mut test_client,
                            &user_token_2022_token_account,
                        )
                        .await;

                        assert_with_msg(
                            user_token_2022_after_deposit_balance == amount_2022 - amount_wrapper,
                            "Invalid user Token2022 token after deposit balance change",
                        );

                        assert_with_msg(
                            user_wrapper_token_after_deposit_balance == amount_wrapper_burn,
                            "Invalid user wrapper token after deposit balance change",
                        );

                        let user_2_wrapper_token_account = create_associated_token_account(
                            &mut test_client,
                            &user_2.pubkey(),
                            &wrapper_token_mint,
                            &spl_token::id(),
                        )
                        .await
                        .unwrap();

                        let transfer_ix = spl_token::instruction::transfer_checked(
                            &spl_token::id(),
                            &user_wrapper_token_account,
                            &wrapper_token_mint,
                            &user_2_wrapper_token_account,
                            &user.pubkey(),
                            &[&user.pubkey()],
                            amount_wrapper_burn,
                            decimal_2022,
                        )
                        .unwrap();

                        let _ = sign_send_instructions(
                            &mut test_client,
                            &[transfer_ix],
                            vec![&user, &payer_keypair],
                            None,
                        )
                        .await;

                        let user_wrapper_token_after_transfer_balance =
                            get_token_balance(&mut test_client, &user_wrapper_token_account).await;

                        let user_2_wrapper_token_after_transfer_balance =
                            get_token_balance(&mut test_client, &user_2_wrapper_token_account)
                                .await;

                        assert_with_msg(
                            user_wrapper_token_after_transfer_balance == 0,
                            "Invalid user wrapper token after transfer balance change",
                        );

                        assert_with_msg(
                            user_2_wrapper_token_after_transfer_balance == amount_wrapper_burn,
                            "Invalid user 2 wrapper token after transfer balance change",
                        );

                        let user_2_token_2022_token_account = create_token_account_token_2022(
                            &mut test_client,
                            &user_2.pubkey(),
                            &token_2022_mint,
                        )
                        .await
                        .unwrap();

                        let burn_ix = create_withdraw_and_burn_wrapper_tokens_instruction(
                            &user_2.pubkey(),
                            &token_2022_mint,
                            &user_2_wrapper_token_account,
                            &user_2_token_2022_token_account,
                            amount_wrapper_burn,
                        );

                        let _ = match sign_send_instructions(
                            &mut test_client,
                            &vec![burn_ix],
                            vec![&user_2, &payer_keypair],
                            None,
                        )
                        .await
                        {
                            Ok(_) => {
                                let user_2_token_2022_after_burn_balance = get_token_balance_2022(
                                    &mut test_client,
                                    &user_2_token_2022_token_account,
                                )
                                .await;

                                let user_2_wrapper_token_after_burn_balance = get_token_balance(
                                    &mut test_client,
                                    &user_2_wrapper_token_account,
                                )
                                .await;

                                assert_with_msg(
                                    user_2_token_2022_after_burn_balance == 98,
                                    "Invalid user 2 Token2022 token after burn balance change",
                                );
                                assert_with_msg(
                                    user_2_wrapper_token_after_burn_balance == 0,
                                    "Invalid user 2 wrapper token after burn balance change",
                                );
                            }
                            Err(e) => {
                                println!("Error burn tx: {}", e);
                            }
                        };
                    }
                    Err(e) => {
                        println!("Error minting wrapper tokens: {}", e);
                    }
                };
            }
            Err(e) => {
                println!("Error initializing token mint: {}", e);
                panic!("test_18 error: {}", e);
            }
        };
    }
}

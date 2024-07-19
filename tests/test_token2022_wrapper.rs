pub mod utils;

use crate::utils::TestClient;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey, signature::Keypair, signer::Signer};
use token2022_wrapper::{error::TokenWrapperError, instruction_builders::create_initialize_wrapper_token_instruction, utils::{get_token_freeze_authority, get_token_mint_authority, get_wrapper_token_mint}};
use utils::{
    airdrop, assert_with_msg, create_associated_token_account, create_mint, create_token_2022_mint, create_token_account_token_2022, extract_error_code, get_token_mint, mint_token_2022_tokens, mint_tokens, sign_send_instructions
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
) -> (Pubkey, Pubkey) {
    let payer_keypair = client.get_payer_clone();

    let token_mint = create_token_2022_mint(client, &payer_keypair.pubkey(), None, decimals, None)
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
            let (expected_mint_authority, _, _) = get_token_mint_authority(wrapper_token, PROGRAM_ID);
            let (expected_freeze_authority, _, _) = get_token_freeze_authority(wrapper_token, PROGRAM_ID);

            let wrapper_token_ac = get_token_mint(&mut test_client, &wrapper_token).await.unwrap();
            
            assert_with_msg(expected_mint_authority == wrapper_token_ac.mint_authority.unwrap(), "Mint authority for the wrapper token does not match");
            assert_with_msg(expected_freeze_authority == wrapper_token_ac.freeze_authority.unwrap(), "Freeze authority for the wrapper token does not match");
            assert_with_msg(wrapper_token_ac.decimals == decimal_2022, "Decimals for the wrapper token does not match");
            assert_with_msg(wrapper_token_ac.supply == 0, "Invalid initial supply for the wrapper token");
        }
        Err(e) => {
            println!("Error: {}", e);
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
                    assert_with_msg(error_code == TokenWrapperError::UnexpectedInitializedAccount as u32, format!("Invalid error thrown for test_2: {}", e).as_str());
                },
                None => {
                    println!("Could not parse error code from the BanksClientError");
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

    let (token_mint, _) = create_and_mint_tokens(
        &mut test_client,
        &user.pubkey(),
        amount,
        decimal,
    )
    .await;

    let token_data = get_token_mint(&mut test_client, &token_mint)
        .await
        .unwrap();

    assert_with_msg(
        token_data.decimals == decimal,
        "Invalid token decimals",
    );

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
            assert_with_msg(e.to_string().contains("incorrect program id"), "Expected test_3 to fail with incorrect program id");
        }
    };
}

// Test 4 - mint test tokens with decimal 5

// Test 5 - mint test tokens with decimal 8

// Test 6 - mint test tokens with decimal 1

// Test 7 - mint test tokens with decimal 0

// Test 8 - works if user A mints, sends it to user B and user B withdraws

// Test 9 - cannot mint if not owned

// Test 10 - cannot mint with an invalid token deposit

// Test 11 - cannot mint if tokens are frozen

// Test 12 - cannot mint if max supply is reached

// Test 13 - burn test tokens with decimal 5

// Test 14 - burn test tokens with decimal 8

// Test 15 - burn test tokens with decimal 1

// Test 16 - burn test tokens with decimal 0

// Test 17 - cannot burn if not owned

// Test 18 - cannot burn an invalid token22 deposit

// Test 19 - cannot burn if tokens are frozen

// Test 20 - cannot burn if max supply is reached
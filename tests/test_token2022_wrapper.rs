pub mod utils;

use crate::utils::TestClient;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer};
use token2022_wrapper::instruction_builders::create_initialize_token_instruction;
use utils::{
    airdrop, assert_with_msg, create_associated_token_account, create_mint, create_token_2022_mint,
    create_token_account_token_2022, get_token_balance, get_token_mint, mint_token_2022_tokens,
    mint_tokens, sign_send_instructions,
};

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

/// Test 1 - testing successful initialization of a vanilla token mint for a Token 2022 mint
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

    let (token_2022_mint, user_token_2022_account) = create_and_mint_tokens_token_2022(
        &mut test_client,
        &user.pubkey(),
        amount_2022,
        decimal_2022,
    )
    .await;

    let user_token_2022_balance =
        get_token_balance(&mut test_client, &user_token_2022_account).await;

    println!("amount_2022: {}", amount_2022);
    println!("user_token_2022_balance: {}", user_token_2022_balance);

    assert_with_msg(
        amount_2022 == user_token_2022_balance,
        "Invalid user token_2022 balance",
    );

    let token_2022_data = get_token_mint(&mut test_client, &token_2022_mint)
        .await
        .unwrap();

    assert_with_msg(
        token_2022_data.decimals == decimal_2022,
        "Invalid token_2022 decimals",
    );

    let initialize_ix =
        create_initialize_token_instruction(&payer_keypair.pubkey(), &token_2022_mint);

    let _ = match sign_send_instructions(
        &mut test_client,
        &vec![initialize_ix],
        vec![&payer_keypair],
        None,
    )
    .await
    {
        Ok(sig) => {
            println!("Initialize wrapper: {:?}", sig);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    };
}

// cannot initialize on repeated tokens

// cannot initialize for an spl token

// mint test tokens with decimal 5

// mint test tokens with decimal 8

// mint test tokens with decimal 1

// mint test tokens with decimal 0

// works if user A mints, sends it to user B and user B withdraws

// cannot mint if not owned

// cannot mint with an invalid token deposit

// cannot mint if tokens are frozen

// cannot mint if max supply is reached

// burn test tokens with decimal 5

// burn test tokens with decimal 8

// burn test tokens with decimal 1

// burn test tokens with decimal 0

// cannot burn if not owned

// cannot burn an invalid token22 deposit

// cannot burn if tokens are frozen

// cannot burn if max supply is reached

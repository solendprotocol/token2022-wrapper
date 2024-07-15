pub mod utils;

use solana_program_test::*;
use solana_sdk::signer::Signer;
use utils::{create_associated_token_account, create_mint, create_token_2022_mint};
use crate::utils::TestClient;

/// Test 1 - testing successful initialization of a vanilla token mint for a Token 2022 mint
/// 
/// 
#[tokio::test]
async fn test_1() {
    let mut test_client = TestClient::new().await;

    let payer_keypair = test_client.get_payer_clone();

    let mint = create_mint(&mut test_client, &payer_keypair.pubkey(), None, 8, None).await.unwrap();

    let create_res = create_associated_token_account(
        &mut test_client,
        &payer_keypair.pubkey(),
        &mint,
        &spl_token::id()
    ).await;

    let mint_2022 = create_token_2022_mint(&mut test_client, &payer_keypair.pubkey(), None, 8, None).await.unwrap();

    let create_res = create_associated_token_account(
        &mut test_client,
        &payer_keypair.pubkey(),
        &mint_2022,
        &spl_token_2022::id()
    ).await;
}
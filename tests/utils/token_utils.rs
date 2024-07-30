use crate::utils::{TestClient, TransactionResult};
use solana_program::pubkey::Pubkey;
use solana_sdk::program_pack::Pack;
use solana_sdk::signature::Signer;
use solana_sdk::{signature::Keypair, system_instruction};
use spl_token::state::Mint;
use super::{get_account, rent_exempt, sign_send_instructions};

pub async fn create_associated_token_account(
    client: &mut TestClient,
    wallet: &Pubkey,
    token_mint: &Pubkey,
    token_program: &Pubkey,
) -> TransactionResult<Pubkey> {
    let ixs = vec![
        spl_associated_token_account::instruction::create_associated_token_account(
            &client.payer.pubkey(),
            wallet,
            token_mint,
            token_program,
        ),
    ];

    let ata_addr = spl_associated_token_account::get_associated_token_address(wallet, token_mint);

    let res = match sign_send_instructions(
        client,
        &ixs,
        vec![&client.get_payer_clone()],
        Some(
            format!(
                "Creating associated token account: {}",
                ata_addr.to_string()
            )
            .as_str(),
        ),
    )
    .await
    {
        Ok(_) => {
            return Ok(ata_addr);
        }
        Err(e) => {
            println!("Error creating ata: {:?}", e);
            Err(e)
        }
    };

    res
}

pub async fn get_balance(client: &mut TestClient, pubkey: &Pubkey) -> u64 {
    client.banks_client.get_balance(*pubkey).await.unwrap()
}

pub async fn get_token_balance(client: &mut TestClient, token_account: &Pubkey) -> u64 {
    let amount = match get_token_account(client, token_account).await {
        Ok(acc) => acc.amount,
        Err(_) => 0,
    };
    amount
}

pub async fn get_token_account(
    client: &mut TestClient,
    token_account: &Pubkey,
) -> TransactionResult<spl_token::state::Account> {
    let account = get_account(client, token_account).await;
    Ok(spl_token::state::Account::unpack(&account.data).unwrap_or_default())
}

pub async fn get_token_mint(
    client: &mut TestClient,
    token_mint: &Pubkey,
) -> TransactionResult<spl_token::state::Mint> {
    let account = get_account(client, token_mint).await;

    Ok(spl_token::state::Mint::unpack(&account.data.split_at(Mint::LEN).0).unwrap())
}

pub async fn create_mint(
    client: &mut TestClient,
    authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    mint: Option<Keypair>,
) -> TransactionResult<Pubkey> {
    let mint = mint.unwrap_or_else(Keypair::new);

    let ixs = vec![
        system_instruction::create_account(
            &client.payer.pubkey(),
            &mint.pubkey(),
            rent_exempt(Mint::LEN),
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint.pubkey(),
            authority,
            freeze_authority,
            decimals,
        )
        .unwrap(),
    ];

    let client_keypair = client.get_payer_clone();

    let token_mint = mint.pubkey();

    let res = match sign_send_instructions(
        client,
        &ixs,
        vec![&client_keypair, &mint],
        Some(format!("Creating mint: {}", token_mint.to_string()).as_str()),
    )
    .await
    {
        Ok(_) => Ok(mint.pubkey()),
        Err(e) => {
            println!("Error creating mint: {}, {:?}", &mint.pubkey(), e);
            Err(e)
        }
    };

    res
}

pub async fn mint_tokens(
    client: &mut TestClient,
    authority: &Keypair,
    mint: &Pubkey,
    account: &Pubkey,
    amount: u64,
    additional_signer: Option<&Keypair>,
) -> TransactionResult<()> {
    let client_keypair = client.get_payer_clone();

    let mut signing_keypairs = vec![&client_keypair, authority];
    if let Some(signer) = additional_signer {
        signing_keypairs.push(signer);
    }

    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        account,
        &authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    let res = match sign_send_instructions(
        client,
        &vec![ix],
        signing_keypairs,
        Some(
            format!(
                "Minting tokens {} to {}",
                mint.to_string(),
                account.to_string()
            )
            .as_str(),
        ),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error minting token {} to {}", mint, account);
            Err(e)
        }
    };

    res
}
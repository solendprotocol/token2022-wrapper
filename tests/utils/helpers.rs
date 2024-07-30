use crate::utils::{TestClient, TransactionResult};
use solana_program::pubkey::Pubkey;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::signature::{Signature, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::{account::Account, rent::Rent, signature::Keypair, system_instruction};
use spl_token::state::Mint;
use spl_token_2022::extension::StateWithExtensionsOwned;
use spl_token_2022::{
    state::Mint as Token2022Mint,
    state::Account as Token2022Account,
    extension::ExtensionType
};
use spl_token_client::token::ExtensionInitializationParams;

use super::TransferFeeConfigWithKeypairs;
use spl_token_2022::extension::BaseStateWithExtensions;

pub async fn sign_send_instructions(
    client: &mut TestClient,
    ixs: &[Instruction],
    signers: Vec<&Keypair>,
    label: Option<&str>,
) -> TransactionResult<Vec<Signature>> {
    let mut transaction = Transaction::new_with_payer(&ixs, Some(&client.payer.pubkey()));
    transaction.sign(
        &signers,
        client.banks_client.get_latest_blockhash().await.unwrap(),
    );

    if label.is_some() {
        // println!("Processing transaction: {:?}", label);
    }

    let signatures = transaction.signatures.clone();

    let res = match client.banks_client.process_transaction(transaction).await {
        Ok(_) => {
            return Ok(signatures);
        }
        Err(e) => {
            println!("Error: {:?}", e);
            Err(e)
        }
    };

    res
}

pub async fn get_account(client: &mut TestClient, pubkey: &Pubkey) -> Account {
    let acc = match client.banks_client.get_account(*pubkey).await {
        Ok(ac) => ac,
        Err(_) => None,
    };

    acc.unwrap_or_default()
}

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

pub async fn create_token_account_token_2022(
    client: &mut TestClient,
    owner: &Pubkey,
    token_mint: &Pubkey,
) -> TransactionResult<Pubkey> {
    let account = Keypair::new();

    let token_mint_info = client.banks_client.get_account(*token_mint).await?.unwrap();
    let token_mint_data_with_extensions = StateWithExtensionsOwned::<Token2022Mint>::unpack(token_mint_info.data).unwrap();
    
    let mint_extensions: Vec<ExtensionType> = token_mint_data_with_extensions.get_extension_types().unwrap();
    let required_extensions =
            ExtensionType::get_required_init_account_extensions(&mint_extensions);

    let space = ExtensionType::try_calculate_account_len::<Token2022Account>(&required_extensions).unwrap();

    let rent = client.banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(space);

    let mut ixs = vec![
        system_instruction::create_account(
            &client.payer.pubkey(),
            &account.pubkey(),
            account_rent,
            space as u64,
            &spl_token_2022::id(),
        ),
    ];

    if required_extensions.contains(&ExtensionType::ImmutableOwner) {
        ixs.push(
            spl_token_2022::instruction::initialize_immutable_owner(
                &spl_token_2022::id(),
                &account.pubkey(),
            ).unwrap()
        )
    }

    ixs.push(
        spl_token_2022::instruction::initialize_account(
            &spl_token_2022::id(),
            &account.pubkey(),
            token_mint,
            owner,
        )
        .unwrap(),
    );

    let res = match sign_send_instructions(
        client,
        &ixs,
        vec![&client.get_payer_clone(), &account],
        Some(
            format!(
                "Creating associated token account: {}",
                &account.pubkey().to_string()
            )
            .as_str(),
        ),
    )
    .await
    {
        Ok(_) => {
            return Ok(account.pubkey());
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

pub async fn get_token_balance_2022(client: &mut TestClient, token_account: &Pubkey) -> u64 {
    let amount = match get_token_account_2022(client, token_account).await {
        Ok(acc) => {
            acc.base.amount
        },
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

pub async fn get_token_account_2022(
    client: &mut TestClient,
    token_account: &Pubkey,
) -> TransactionResult<StateWithExtensionsOwned<Token2022Account>> {
    let account_info = get_account(client, token_account).await;
    let account: StateWithExtensionsOwned<Token2022Account> = StateWithExtensionsOwned::<Token2022Account>::unpack(account_info.data).unwrap();

    Ok(account)
}

pub async fn get_token_mint(
    client: &mut TestClient,
    token_mint: &Pubkey,
) -> TransactionResult<spl_token::state::Mint> {
    let account = get_account(client, token_mint).await;

    Ok(spl_token::state::Mint::unpack(&account.data.split_at(Mint::LEN).0).unwrap())
}

pub async fn get_token_mint_2022(
    client: &mut TestClient,
    token_mint: &Pubkey,
) -> TransactionResult<StateWithExtensionsOwned<Token2022Mint>> {
    let token_mint_info = client.banks_client.get_account(*token_mint).await?.unwrap();
    let token_mint_data_with_extensions: StateWithExtensionsOwned<Token2022Mint> = StateWithExtensionsOwned::<Token2022Mint>::unpack(token_mint_info.data).unwrap();

    Ok(token_mint_data_with_extensions)
}


pub async fn airdrop(
    client: &mut TestClient,
    receiver: &Pubkey,
    amount: u64,
) -> TransactionResult<()> {
    let ixs = vec![system_instruction::transfer(
        &client.payer.pubkey(),
        receiver,
        amount,
    )];

    let res = match sign_send_instructions(
        client,
        &ixs,
        vec![&client.get_payer_clone()],
        Some(format!("Airdropping {} SOL to {}", amount, receiver.to_string()).as_str()),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {:?}", e);
            Err(e)
        }
    };

    res
}

pub fn rent_exempt(size: usize) -> u64 {
    Rent::default().minimum_balance(size) as u64
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

pub async fn create_token_2022_mint(
    client: &mut TestClient,
    authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    mint: Option<Keypair>,
    transfer_fee_config: Option<&TransferFeeConfigWithKeypairs>,
) -> TransactionResult<Pubkey> {
    let mint = mint.unwrap_or_else(Keypair::new);

    let mut extension_params = vec![];

    if transfer_fee_config.is_some() {
        let config = transfer_fee_config.unwrap();

        let transfer_fee_basis_points = u16::from(
            config
                .transfer_fee_config
                .newer_transfer_fee
                .transfer_fee_basis_points,
        );

        let maximum_fee = u64::from(
            config
                .transfer_fee_config
                .newer_transfer_fee
                .maximum_fee
        );

        extension_params.push(
            ExtensionInitializationParams::TransferFeeConfig { 
                transfer_fee_config_authority: Some(config.transfer_fee_config_authority.pubkey()),
                withdraw_withheld_authority: Some(config.withdraw_withheld_authority.pubkey()),
                transfer_fee_basis_points,
                maximum_fee
            }
        );
    }

    let extension_types = extension_params
        .iter()
        .map(|e| e.extension())
        .collect::<Vec<_>>();
    let space = ExtensionType::try_calculate_account_len::<Token2022Mint>(&extension_types).unwrap();    
    let mut ixs = vec![
        system_instruction::create_account(
            &client.payer.pubkey(),
            &mint.pubkey(),
            rent_exempt(space),
            space as u64,
            &spl_token_2022::id(),
        )
    ];

    for extension in extension_params {
        ixs.push(extension.instruction(&spl_token_2022::id(), &mint.pubkey()).unwrap());
    }

    ixs.push(
        spl_token_2022::instruction::initialize_mint2(
            &spl_token_2022::id(),
            &mint.pubkey(),
            authority,
            freeze_authority,
            decimals,
        )
        .unwrap()
    );

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
            println!("Error creating Token 2022 mint: {}", &mint.pubkey());
            Err(e)
        }
    };

    res
}

pub async fn mint_token_2022_tokens(
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

    let token_data = get_token_mint_2022(client, mint).await;

    let decimals = match token_data {
        Ok(dec) => {
            dec.base.decimals
        },
        Err(_) => std::u8::MAX,
    };

    if decimals == std::u8::MAX {
        return Err(solana_program_test::BanksClientError::Io(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid decimals"),
        ));
    }

    let ix = spl_token_2022::instruction::mint_to_checked(
        &spl_token_2022::id(),
        mint,
        account,
        &authority.pubkey(),
        &[&authority.pubkey()],
        amount,
        decimals,
    )
    .unwrap();

    let res = match sign_send_instructions(
        client,
        &vec![ix],
        signing_keypairs,
        Some(
            format!(
                "Minting token {} to {}",
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
            println!(
                "Token 2022 --> Error minting {} to {}, {}",
                &mint, &account, e
            );
            Err(e)
        }
    };

    res
}

pub fn assert_with_msg(v: bool, msg: &str) {
    if !v {
        let caller = std::panic::Location::caller();
        println!("{}. \n{}", msg, caller);
    }
}

pub fn extract_error_code(error_message: &str) -> Option<u32> {
    if let Some(start_index) = error_message.find("0x") {
        let error_code_str = &error_message[start_index + 2..];
        if let Some(end_index) = error_code_str.find(|c: char| !c.is_digit(16)) {
            let error_code_hex = &error_code_str[..end_index];
            if let Ok(error_code) = u32::from_str_radix(error_code_hex, 16) {
                return Some(error_code);
            }
        } else if let Ok(error_code) = u32::from_str_radix(error_code_str, 16) {
            return Some(error_code);
        }
    }
    None
}

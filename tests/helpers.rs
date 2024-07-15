use std::collections::HashSet;
use solana_program_test::{processor, BanksClient, ProgramTest};
use spl_token::state::Mint;
use thiserror::Error;
use solana_program::{
    pubkey::Pubkey,
    program_error::ProgramError
};
use solana_sdk::{
    account::Account, instruction::Instruction, rent::Rent, signature::{Keypair, Signature}, system_instruction, transaction::Transaction, transport::TransportError
};
use solana_sdk::signature::Signer;
use itertools::Itertools;
use solana_sdk::program_pack::Pack;

pub type TransactionResult<T = ()> = std::result::Result<T, TransactionError>;

#[derive(Error, Debug)]
pub enum TransactionError {
#[error("Missing signer for transaction")]
MissingSigner { signer: Pubkey },
#[error("Transaction timed out")]
TransactionTimeout { elapsed_ms: u64 },
#[error("Action is not suppported")]
UnsupportedAction,
#[error("Solana client error")]
SolanaClient(#[from] solana_client::client_error::ClientError),
#[error("Some other error")]
Other(#[from] anyhow::Error),
#[error("Transaction Failed")]
TransactionFailed {
    signature: Signature,
    logs: Vec<String>,
},
#[error("Transport Error")]
TransportError(#[from] TransportError),
#[error("Program Error")]
ProgramError(#[from] ProgramError),
}

pub struct TestClient {
    pub banks_client: BanksClient,
    pub payer: Keypair,
}

impl TestClient {
    pub async fn new() -> TestClient {
        let program = ProgramTest::new(
            "token2022_wrapper",
            token2022_wrapper::id(),
            processor!(token2022_wrapper::processor::process_instruction)
        );

        let (client, payer, _) = program.start().await;

        TestClient {
            banks_client: client,
            payer
        }
    }

    /// UNSAFE, only for tests
    pub fn get_payer_clone(&self) -> Keypair {
        self.payer.insecure_clone()
    }

    pub async fn sign_send_instructions(
        &mut self,
        instructions: Vec<Instruction>,
        mut signers: Vec<&Keypair>,
    ) -> Result<(), TransactionError> {
        let required_signers = instructions
            .iter()
            .flat_map(|i| {
                i.accounts
                    .iter()
                    .filter_map(|am| if am.is_signer { Some(am.pubkey) } else { None })
                    .collect::<Vec<Pubkey>>()
            })
            .unique()
            .collect::<Vec<Pubkey>>();

        let existing_signers = signers
            .iter()
            .map(|k| k.pubkey())
            .unique()
            .collect::<HashSet<Pubkey>>();

        for required_signer in required_signers.iter() {
            if !existing_signers.contains(required_signer) {
                return Err(TransactionError::MissingSigner {
                    signer: *required_signer,
                });
            }
        }

        let payer = {
            signers.retain(|k| k.pubkey() != self.payer.pubkey());
            signers.insert(0, &self.payer);
            self.payer.pubkey()
        };

        let tx = Transaction::new_with_payer(&instructions, Some(&payer));

        let _ = self.banks_client.send_transaction(tx).await;

        Ok(())
    }
}

pub async fn get_account(client: &mut TestClient, pubkey: &Pubkey) -> Account {
    client.banks_client.get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
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
    client.sign_send_instructions(ixs, vec![]).await?;

    Ok(spl_associated_token_account::get_associated_token_address(
        wallet, token_mint,
    ))
}

pub async fn get_balance(client: &mut TestClient, pubkey: &Pubkey) -> u64 {
    client.banks_client.get_balance(*pubkey).await.unwrap()
}

pub async fn get_token_balance(client: &mut TestClient, token_account: &Pubkey) -> u64 {
    get_token_account(client, token_account)
        .await
        .unwrap()
        .amount
}

pub async fn get_token_account(
    client: &mut TestClient,
    token_account: &Pubkey,
) -> TransactionResult<spl_token::state::Account> {
    let account = get_account(client, token_account).await;
    Ok(spl_token::state::Account::unpack(&account.data).unwrap())
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

    client.sign_send_instructions(ixs, vec![]).await
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
) -> TransactionResult<Keypair> {
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

    client
        .sign_send_instructions(ixs, vec![&client_keypair, &mint])
        .await
        .unwrap();
    Ok(mint)
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

    client
        .sign_send_instructions(vec![ix], signing_keypairs)
        .await
}

use std::collections::HashSet;
use solana_program_test::{processor, BanksClient, ProgramTest};
use thiserror::Error;
use solana_program::{
    pubkey::Pubkey,
    program_error::ProgramError
};
use solana_sdk::{
    instruction::Instruction, signature::{Keypair, Signature}, transaction::Transaction, transport::TransportError
};
use solana_sdk::signature::Signer;
use itertools::Itertools;

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

use solana_program_test::{processor, BanksClient, ProgramTest};
use thiserror::Error;
use solana_program::{
    pubkey::Pubkey,
    program_error::ProgramError
};
use solana_sdk::{ signature::{Keypair, Signature}, transport::TransportError
};

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
#[error("Invalid data")]
InvalidData
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
}

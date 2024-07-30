use crate::utils::{TestClient, TransactionResult};
use solana_program::pubkey::Pubkey;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::{Signature, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::{account::Account, rent::Rent, signature::Keypair, system_instruction};

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

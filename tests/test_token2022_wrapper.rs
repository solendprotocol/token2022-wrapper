pub mod helpers;

#[cfg(test)]
mod tests {

    use solana_program_test::*;
    use crate::helpers::TestClient;

    #[tokio::test]
    async fn sample_test() {
        let mut test_client = TestClient::new().await;

        let payer_keypair = test_client.get_payer_clone();

        let _ = test_client.sign_send_instructions(vec![], vec![&payer_keypair]);
    }
}
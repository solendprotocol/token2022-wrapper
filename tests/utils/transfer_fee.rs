use solana_sdk::{program_option::COption, signature::Keypair, signer::Signer};
use spl_token_2022::extension::transfer_fee::{TransferFee, TransferFeeConfig};

const TEST_MAXIMUM_FEE: u64 = 10_000_000;
const TEST_TRANSFER_FEE_IN_BASIS_POINTS: u16 = 100;

pub fn test_transfer_fee() -> TransferFee {
    TransferFee {
        epoch: 0.into(),
        transfer_fee_basis_points: TEST_TRANSFER_FEE_IN_BASIS_POINTS.into(),
        maximum_fee: TEST_MAXIMUM_FEE.into(),
    }
}

pub struct TransferFeeConfigWithKeypairs {
    pub transfer_fee_config: TransferFeeConfig,
    pub transfer_fee_config_authority: Keypair,
    pub withdraw_withheld_authority: Keypair
}

pub fn test_transfer_fee_config_with_keypairs() -> TransferFeeConfigWithKeypairs {
    let transfer_fee = test_transfer_fee();
    let transfer_fee_config_authority = Keypair::new();
    let withdraw_withheld_authority = Keypair::new();

    let transfer_fee_config = TransferFeeConfig {
        transfer_fee_config_authority: COption::Some(transfer_fee_config_authority.pubkey()).try_into().unwrap(),
        withdraw_withheld_authority: COption::Some(withdraw_withheld_authority.pubkey()).try_into().unwrap(),
        withheld_amount: 0.into(),
        older_transfer_fee: transfer_fee,
        newer_transfer_fee: transfer_fee
    };
    
    TransferFeeConfigWithKeypairs {
        transfer_fee_config,
        transfer_fee_config_authority,
        withdraw_withheld_authority
    }
}
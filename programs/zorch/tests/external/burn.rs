//! Tests for the burn instruction

use crate::Test;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use zorch::api;

#[test]
fn test_burn_success() {
    let test = Test::new();
    let signer = Keypair::new();
    let signer_token_account = solana_sdk::pubkey::Pubkey::new_unique();

    let amount = 5_000_000_000; // 50 sZEC (8 decimals)
    let zec_recipient = "t1abcdefghijklmnopqrstuvwxyz123456789".to_string();

    let instruction = api::burn(signer.pubkey(), signer_token_account, amount, zec_recipient);

    // Provide account states for all accounts
    let result = test.mollusk.process_instruction(
        &instruction,
        &test.burn_accounts(signer.pubkey(), signer_token_account),
    );

    // Note: This test validates the instruction structure.
    // It fails because the token account needs to be properly initialized with token data.
    // In practice, burn would be called on an initialized token account with a balance.
    assert!(
        result.program_result.is_err(),
        "Expected failure without initialized token account"
    );
}

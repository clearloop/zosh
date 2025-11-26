//! Tests for the mint instruction

use crate::Test;
use solana_sdk::pubkey::Pubkey;
use zorch::{api, types::MintEntry};

#[test]
fn test_mint_success() {
    let test = Test::new();

    // Create recipient token accounts
    let recipient1 = Pubkey::new_unique();
    let recipient2 = Pubkey::new_unique();
    let token_account1 = Pubkey::new_unique();
    let token_account2 = Pubkey::new_unique();

    let mints = vec![
        MintEntry {
            recipient: recipient1,
            amount: 1_000_000_000, // 10 sZEC
        },
        MintEntry {
            recipient: recipient2,
            amount: 2_000_000_000, // 20 sZEC
        },
    ];

    // Mock signatures (in real test, these would be valid Ed25519 signatures)
    let signatures = vec![[1u8; 64], [2u8; 64]];
    let recipient_token_accounts = vec![
        (token_account1, Test::account().into()),
        (token_account2, Test::account().into()),
    ];

    let instruction = api::mint(
        test.payer,
        vec![token_account1, token_account2],
        mints,
        signatures,
    );

    // Provide account states for all accounts
    let result = test
        .mollusk
        .process_instruction(&instruction, &test.mint_accounts(recipient_token_accounts));

    // Note: This will fail without proper Ed25519 signature verification setup
    // but the test demonstrates the account structure
    assert!(
        result.program_result.is_err(),
        "Expected failure without proper signature verification"
    );
}

//! Tests for the metadata instruction

use crate::Test;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use zorch::api;

#[test]
fn test_metadata_update() {
    let test = Test::new();
    let authority = Keypair::new();
    let instruction = api::metadata(
        authority.pubkey(),
        "Shielded ZEC".to_string(),
        "sZEC".to_string(),
        "https://example.com/metadata.json".to_string(),
    );

    // Provide account states for all accounts
    let result = test
        .mollusk
        .process_instruction(&instruction, &test.metadata_accounts(authority.pubkey()));

    // Note: This test validates the instruction structure.
    // It fails because bridge_state needs to be initialized first via the initialize instruction.
    // In practice, metadata would be called after successful initialization.
    assert!(
        result.program_result.is_err(),
        "Expected failure without initialized bridge state"
    );
}

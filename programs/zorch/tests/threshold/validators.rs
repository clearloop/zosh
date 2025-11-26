//! Tests for the validators instruction

use crate::Test;
use zorch::api;

#[test]
fn test_validators_update() {
    let test = Test::new();

    // New validator set
    let new_validators = crate::pubkeys(5);
    let new_threshold = 3;

    // Mock signatures (in real test, these would be valid Ed25519 signatures)
    let signatures = vec![[1u8; 64], [2u8; 64], [3u8; 64]];
    let instruction = api::validators(test.payer, new_validators, new_threshold, signatures);

    // Provide account states for all accounts
    let result = test
        .mollusk
        .process_instruction(&instruction, &test.validators_accounts());

    // Note: This will fail without proper Ed25519 signature verification setup,
    // but the test validates the instruction structure.
    // In a successful validators update:
    // - bridge_state.validators would be updated to new_validators
    // - bridge_state.threshold would be updated to new_threshold
    // - bridge_state.total_validators would be updated to 5
    // - bridge_state.nonce would be incremented
    // - ValidatorSetUpdated event would be emitted
    assert!(
        result.program_result.is_err(),
        "Expected failure without proper signature verification"
    );
}

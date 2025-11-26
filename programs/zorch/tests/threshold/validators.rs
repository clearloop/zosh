//! Tests for the validators instruction

use crate::Test;
use zorch::api;

#[test]
fn test_validators_update() {
    let test = Test::new();

    // 1. initialize the bridge state
    let instruction = api::initialize(test.payer, vec![test.payer], 1);
    let result = test
        .mollusk
        .process_instruction(&instruction, &test.initialize_accounts());
    assert!(!result.program_result.is_err());

    // 2. process the ed25519 instruction
    let (signature, instruction) = test.validators_signatures(0, vec![test.payer], 1);
    let result = test.mollusk.process_instruction(
        &instruction,
        &vec![
            (test.payer, Test::account().into()),
            (solana_sdk::ed25519_program::ID, Test::account().into()),
        ],
    );
    assert!(!result.program_result.is_err());

    // 3. process the validators instruction
    let instruction = api::validators(test.payer, vec![test.payer], 1, vec![signature]);
    let result = test
        .mollusk
        .process_instruction(&instruction, &test.validators_accounts());

    assert!(
        !result.program_result.is_err(),
        "Program execution failed: {:?}",
        result.program_result
    );
}

//! Tests for the validators instruction

use crate::Test;
use zorch::api;

#[test]
fn test_validators_update() {
    let test = Test::new();
    let (signature, sig_instruction) = test.validators_signatures(0, vec![test.payer], 1);
    let instruction = api::validators(test.payer, vec![test.payer], 1, vec![signature]);
    let result = test
        .mollusk
        .process_instruction_chain(&[sig_instruction, instruction], &test.validators_accounts());

    assert!(
        !result.program_result.is_err(),
        "Program execution failed: {:?}",
        result.program_result
    );
}

//! Tests for the initialize instruction

use crate::Test;
use zorch::api;

#[test]
fn test_initialize_success() {
    let test = Test::new();
    let vals = crate::pubkeys(3);
    let instruction = api::initialize(test.payer, vals, 2);

    // Provide account states for all accounts
    let result = test
        .mollusk
        .process_instruction(&instruction, &test.initialize_accounts());

    assert!(
        !result.program_result.is_err(),
        "Program execution failed: {:?}",
        result.program_result
    );
}

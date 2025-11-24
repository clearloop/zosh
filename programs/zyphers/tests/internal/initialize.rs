//! Tests for the initialize instruction

use crate::Test;
use zyphers::api;

#[test]
fn test_initialize_success() {
    let test = Test::new();
    let vals = crate::pubkeys(3);
    let instruction = api::initialize(test.payer, vals, 2);

    // Provide account states for all accounts
    let result = test.mollusk.process_instruction(
        &instruction,
        &[
            (test.payer, Test::account().into()),
            (api::pda::bridge_state(), Default::default()),
            (api::pda::szec_mint(), Default::default()),
            (
                api::pda::SYSTEM_PROGRAM,
                Test::native_program_account().into(),
            ),
            (api::pda::TOKEN_PROGRAM, Test::bpf_program_account().into()),
            test.mollusk.sysvars.keyed_account_for_rent_sysvar(),
        ],
    );

    assert!(
        !result.program_result.is_err(),
        "Program execution failed: {:?}",
        result.program_result
    );
}

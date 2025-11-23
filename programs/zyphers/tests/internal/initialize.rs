//! Tests for the initialize instruction

use crate::Test;
use zyphers::api;

#[test]
fn test_initialize_success() {
    let test = Test::new();
    let vals = crate::pubkeys(3);
    let instruction = api::initialize(test.payer, vals, 2);
    let result = test.mollusk.process_instruction(
        &instruction,
        &[
            (test.payer, Test::account().into()),
            (api::pda::bridge_state(), Default::default()),
            (api::pda::szec_mint(), Default::default()),
            (api::pda::SYSTEM_PROGRAM, Default::default()),
            (api::pda::TOKEN_PROGRAM, Default::default()),
            (api::pda::RENT, Default::default()),
        ],
    );
    assert!(!result.program_result.is_err());
}

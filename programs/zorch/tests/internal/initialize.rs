//! Tests for the initialize instruction

use crate::Test;
use anchor_lang::AnchorDeserialize;
use zorch::{api, BridgeState};

#[test]
fn test_initialize_success() {
    let test = Test::new();
    let vals = crate::pubkeys(3);
    let threshold = 2;
    let instruction = api::initialize(test.payer, vals.clone(), threshold);

    // Provide account states for all accounts
    let result = test
        .mollusk
        .process_instruction(&instruction, &test.initialize_accounts());

    assert!(
        !result.program_result.is_err(),
        "Program execution failed: {:?}",
        result.program_result
    );

    // Verify the bridge state was initialized correctly
    let bridge_state_account = result
        .resulting_accounts
        .iter()
        .find(|(key, _)| *key == api::pda::bridge_state())
        .expect("Bridge state account not found");

    let bridge_state_data = &bridge_state_account.1.data;
    let bridge_state =
        BridgeState::deserialize(&mut &bridge_state_data[8..]).expect("Failed to deserialize");
    assert_eq!(bridge_state.authority, test.payer, "Authority mismatch");
    assert_eq!(bridge_state.validators, vals, "Validators mismatch");
    assert_eq!(bridge_state.threshold, threshold, "Threshold mismatch");
    assert_eq!(
        bridge_state.total_validators, 3,
        "Total validators mismatch"
    );
    assert_eq!(bridge_state.nonce, 0, "Initial nonce should be 0");
    assert_eq!(
        bridge_state.zec_mint,
        api::pda::zec_mint(),
        "ZEC mint mismatch"
    );
}

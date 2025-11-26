//! Tests for threshold instructions

use crate::Test;
use solana_sdk::signer::Signer;

mod mint;
mod validators;

#[test]
fn test_ed25519_success() {
    let test = Test::new();
    let message = b"test message";
    let signature = test.pair.sign_message(message);
    let instruction = solana_ed25519_program::new_ed25519_instruction_with_signature(
        message,
        &signature.as_array(),
        test.payer.as_array(),
    );

    let result = test.mollusk.process_instruction(
        &instruction,
        &vec![
            (test.payer, Test::account().into()),
            (solana_sdk::ed25519_program::ID, Test::account().into()),
        ],
    );

    assert!(!result.program_result.is_err());
}

#[test]
fn test_ed25519_failure() {
    let test = Test::new();
    let message = b"test message";
    let signature = test.pair.sign_message(message);
    let instruction = solana_ed25519_program::new_ed25519_instruction_with_signature(
        b"wrong message",
        &signature.as_array(),
        test.payer.as_array(),
    );

    let result = test.mollusk.process_instruction(
        &instruction,
        &vec![
            (test.payer, Test::account().into()),
            (solana_sdk::ed25519_program::ID, Test::account().into()),
        ],
    );

    assert!(result.program_result.is_err());
}

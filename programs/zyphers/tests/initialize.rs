//! Tests for the initialize instruction

use anchor_lang::AnchorSerialize;
use mollusk_svm::Mollusk;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use zyphers::instruction::Initialize;

#[test]
fn test_initialize_success() -> anyhow::Result<()> {
    let mollusk = Mollusk::new(&zyphers::ID, "zyphers");
    let validators = vec![
        Pubkey::new_from_array([0; 32]),
        Pubkey::new_from_array([1; 32]),
        Pubkey::new_from_array([2; 32]),
    ];
    let initialize = Initialize {
        initial_validators: validators,
        threshold: 2,
    };

    // process the transaction
    let authority = Pubkey::new_unique();
    let instruction = Instruction::new_with_bytes(zyphers::ID, &initialize.try_to_vec()?, vec![]);
    let result =
        mollusk.process_instruction(&instruction, &[(authority.clone(), Default::default())]);
    println!("result: {:?}", result);
    Ok(())
}

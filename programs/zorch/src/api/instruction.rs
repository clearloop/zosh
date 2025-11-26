//! instructions for the zorch program

use anchor_lang::{prelude::AccountMeta, InstructionData};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

use crate::api::pda;

/// Build the initialize instruction
pub fn initialize(payer: Pubkey, validators: Vec<Pubkey>, threshold: u8) -> Instruction {
    let data = crate::instruction::Initialize {
        validators,
        threshold,
    };

    // build the instruction accounts
    let accounts = vec![
        AccountMeta::new(payer, true),                // payer (signer, mut)
        AccountMeta::new(pda::bridge_state(), false), // bridge_state (mut)
        AccountMeta::new(pda::zec_mint(), false),     // zec_mint (mut)
        AccountMeta::new_readonly(pda::SYSTEM_PROGRAM, false), // system_program
        AccountMeta::new_readonly(pda::TOKEN_PROGRAM, false), // token_program
        AccountMeta::new_readonly(pda::RENT, false),  // rent sysvar
    ];

    Instruction::new_with_bytes(crate::ID, &data.data(), accounts)
}

/// Build the metadata instruction
pub fn metadata(authority: Pubkey, name: String, symbol: String, uri: String) -> Instruction {
    let data = crate::instruction::Metadata { name, symbol, uri };

    // build the instruction accounts
    let accounts = vec![
        AccountMeta::new(authority, true), // authority (signer, mut)
        AccountMeta::new_readonly(pda::bridge_state(), false), // bridge_state
        AccountMeta::new(pda::zec_mint(), false), // zec_mint (mut)
        AccountMeta::new(pda::metadata(), false), // metadata (mut)
        AccountMeta::new_readonly(pda::TOKEN_METADATA_PROGRAM, false), // token_metadata_program
        AccountMeta::new_readonly(pda::SYSTEM_PROGRAM, false), // system_program
        AccountMeta::new_readonly(pda::RENT, false), // rent sysvar
    ];

    Instruction::new_with_bytes(crate::ID, &data.data(), accounts)
}

/// Build the mint instruction
pub fn mint(
    payer: Pubkey,
    recipient_token_accounts: Vec<Pubkey>,
    mints: Vec<crate::types::MintEntry>,
    signatures: Vec<[u8; 64]>,
) -> Instruction {
    let data = crate::instruction::Mint { mints, signatures };

    // build the instruction accounts
    let mut accounts = vec![
        AccountMeta::new(payer, true),                // payer (signer, mut)
        AccountMeta::new(pda::bridge_state(), false), // bridge_state (mut)
        AccountMeta::new(pda::zec_mint(), false),     // zec_mint (mut)
        AccountMeta::new_readonly(pda::TOKEN_PROGRAM, false), // token_program
        AccountMeta::new_readonly(pda::SYSTEM_PROGRAM, false), // system_program
        AccountMeta::new_readonly(pda::INSTRUCTIONS_SYSVAR, false), // instructions sysvar
    ];

    // add recipient token accounts as remaining accounts
    for token_account in recipient_token_accounts {
        accounts.push(AccountMeta::new(token_account, false));
    }

    Instruction::new_with_bytes(crate::ID, &data.data(), accounts)
}

/// Build the burn instruction
pub fn burn(
    signer: Pubkey,
    signer_token_account: Pubkey,
    amount: u64,
    zec_recipient: String,
) -> Instruction {
    let data = crate::instruction::Burn {
        amount,
        zec_recipient,
    };

    // build the instruction accounts
    let accounts = vec![
        AccountMeta::new(signer, true),                // signer (signer, mut)
        AccountMeta::new(signer_token_account, false), // signer_token_account (mut)
        AccountMeta::new(pda::zec_mint(), false),      // zec_mint (mut)
        AccountMeta::new_readonly(pda::bridge_state(), false), // bridge_state
        AccountMeta::new_readonly(pda::TOKEN_PROGRAM, false), // token_program
    ];

    Instruction::new_with_bytes(crate::ID, &data.data(), accounts)
}

/// Build the validators instruction
pub fn validators(
    payer: Pubkey,
    new_validators: Vec<Pubkey>,
    new_threshold: u8,
    signatures: Vec<[u8; 64]>,
) -> Instruction {
    let data = crate::instruction::Validators {
        new_validators,
        new_threshold,
        signatures,
    };

    // build the instruction accounts
    let accounts = vec![
        AccountMeta::new(payer, true),                // payer (signer, mut)
        AccountMeta::new(pda::bridge_state(), false), // bridge_state (mut)
        AccountMeta::new_readonly(pda::SYSTEM_PROGRAM, false), // system_program
        AccountMeta::new_readonly(pda::INSTRUCTIONS_SYSVAR, false), // instructions sysvar
    ];

    Instruction::new_with_bytes(crate::ID, &data.data(), accounts)
}

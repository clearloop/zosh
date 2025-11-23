//! client library for the zyphers program
#![cfg(not(target_os = "solana"))]

use crate::instruction::Initialize;
use anchor_lang::{prelude::AccountMeta, system_program, AnchorSerialize};
use anchor_spl::token;
use anyhow::Result;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

/// Build the initialize instruction
pub fn initialize(payer: Pubkey, validators: Vec<Pubkey>, threshold: u8) -> Result<Instruction> {
    let initialize = Initialize {
        validators,
        threshold,
    };

    // Derive PDAs
    //
    // TODO: derive PDAs for the token metadata as well
    let (bridge_state, _bump) = Pubkey::find_program_address(&[b"bridge-state"], &crate::ID);
    let (szec_mint, _mint_bump) = Pubkey::find_program_address(&[b"szec-mint"], &crate::ID);

    // build the instruction accounts
    let accounts = vec![
        AccountMeta::new(payer, true),         // payer (signer, mut)
        AccountMeta::new(bridge_state, false), // bridge_state (mut)
        AccountMeta::new(szec_mint, false),    // szec_mint (mut)
        AccountMeta::new_readonly(system_program::ID, false), // system_program
        AccountMeta::new_readonly(token::ID, false), // token_program - using anchor_spl::token::ID
        AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false), // rent sysvar
    ];

    Ok(Instruction::new_with_bytes(
        crate::ID,
        &initialize.try_to_vec()?,
        accounts,
    ))
}

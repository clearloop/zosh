//! client library for the zorch program
#![cfg(not(target_os = "solana"))]

use crate::instruction::Initialize;
use anchor_lang::{prelude::AccountMeta, InstructionData};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

/// Build the initialize instruction
pub fn initialize(payer: Pubkey, validators: Vec<Pubkey>, threshold: u8) -> Instruction {
    let initialize = Initialize {
        validators,
        threshold,
    };

    // build the instruction accounts
    let accounts = vec![
        AccountMeta::new(payer, true),                // payer (signer, mut)
        AccountMeta::new(pda::bridge_state(), false), // bridge_state (mut)
        AccountMeta::new(pda::zec_mint(), false),    // zec_mint (mut)
        AccountMeta::new_readonly(pda::SYSTEM_PROGRAM, false), // system_program
        AccountMeta::new_readonly(pda::TOKEN_PROGRAM, false), // token_program - using anchor_spl::token::ID
        AccountMeta::new_readonly(pda::RENT, false),          // rent sysvar
    ];

    Instruction::new_with_bytes(crate::ID, &initialize.data(), accounts)
}

/// PDA helpers
///
/// TODO: derive PDAs for the token metadata as well
pub mod pda {
    use anchor_lang::system_program;
    use anchor_spl::token;
    use solana_sdk::pubkey::Pubkey;

    /// System program ID
    pub const SYSTEM_PROGRAM: Pubkey = system_program::ID;

    /// Token program ID
    pub const TOKEN_PROGRAM: Pubkey = token::ID;

    /// Rent sysvar ID
    pub const RENT: Pubkey = solana_sdk::sysvar::rent::ID;

    /// Derive the bridge state PDA
    pub fn bridge_state() -> Pubkey {
        Pubkey::find_program_address(&[b"bridge-state"], &crate::ID).0
    }

    /// Derive the sZEC mint PDA
    pub fn zec_mint() -> Pubkey {
        Pubkey::find_program_address(&[b"zec-mint"], &crate::ID).0
    }
}

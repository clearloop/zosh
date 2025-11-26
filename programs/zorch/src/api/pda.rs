//! PDA functions for the zorch program

use anchor_lang::system_program;
use anchor_spl::token;
use solana_sdk::pubkey::Pubkey;

/// System program ID
pub const SYSTEM_PROGRAM: Pubkey = system_program::ID;

/// Token program ID
pub const TOKEN_PROGRAM: Pubkey = token::ID;

/// Rent sysvar ID
pub const RENT: Pubkey = solana_sdk::sysvar::rent::ID;

/// Instructions sysvar ID
pub const INSTRUCTIONS_SYSVAR: Pubkey = solana_sdk::sysvar::instructions::ID;

/// Token metadata program ID
pub const TOKEN_METADATA_PROGRAM: Pubkey = mpl_token_metadata::ID;

/// Derive the bridge state PDA
pub fn bridge_state() -> Pubkey {
    Pubkey::find_program_address(&[b"bridge-state"], &crate::ID).0
}

/// Derive the sZEC mint PDA
pub fn zec_mint() -> Pubkey {
    Pubkey::find_program_address(&[b"zec-mint"], &crate::ID).0
}

/// Derive the metadata PDA for the sZEC mint
pub fn metadata() -> Pubkey {
    let zec_mint = zec_mint();
    Pubkey::find_program_address(
        &[
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            zec_mint.as_ref(),
        ],
        &mpl_token_metadata::ID,
    )
    .0
}

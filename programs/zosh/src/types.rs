//! shared types

use anchor_lang::prelude::*;

/// Mint entry for batch minting
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MintEntry {
    /// Recipient pubkey
    pub recipient: Pubkey,
    /// Amount to mint
    pub amount: u64,
}

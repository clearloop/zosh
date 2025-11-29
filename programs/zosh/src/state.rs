use anchor_lang::prelude::*;

/// The main bridge state account that stores validator set and configuration
#[account]
#[derive(Debug)]
pub struct BridgeState {
    /// Program authority pubkey
    pub authority: Pubkey,

    /// The validators for the consensus of zosh
    pub validators: Vec<Pubkey>,

    /// The threshold for the consensus (e.g., 2 for 2/3)
    pub threshold: u8,

    /// Total number of validators (e.g., 3 for 2/3)
    pub total_validators: u8,

    /// The sZEC SPL token mint
    pub zec_mint: Pubkey,

    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl BridgeState {
    /// Calculate space needed for the account
    pub fn space(num_validators: usize) -> usize {
        8 + // discriminator
        32 + // authority
        4 + (num_validators * 32) + // validators vec
        1 + // threshold
        1 + // total_validators
        32 + // zec_mint
        1 // bump
    }
}

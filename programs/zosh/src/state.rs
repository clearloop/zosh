use anchor_lang::prelude::*;

/// The main bridge state account that stores validator set and configuration
#[account]
#[derive(Debug)]
pub struct BridgeState {
    /// Program authority pubkey
    pub authority: Pubkey,

    /// The sZEC SPL token mint
    pub zec_mint: Pubkey,

    /// The MPC pubkey
    pub mpc: Pubkey,

    /// Bump seed for PDA derivation
    pub bump: u8,
}

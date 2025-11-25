use anchor_lang::prelude::*;

/// The main bridge state account that stores validator set and configuration
#[account]
pub struct BridgeState {
    /// Program authority pubkey
    pub authority: Pubkey,

    /// The validators for the consensus of zorch
    pub validators: Vec<Pubkey>,

    /// The threshold for the consensus (e.g., 2 for 2/3)
    pub threshold: u8,

    /// Total number of validators (e.g., 3 for 2/3)
    pub total_validators: u8,

    /// Nonce for replay protection
    pub nonce: u64,

    /// The sZEC SPL token mint
    pub szec_mint: Pubkey,

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
        8 + // nonce
        32 + // szec_mint
        1 // bump
    }
}

/// Action record to prevent replay attacks
#[account]
pub struct ActionRecord {
    /// Hash of the action
    pub action_hash: [u8; 32],

    /// Whether the action was executed
    pub executed: bool,

    /// Validators who signed this action
    pub signers: Vec<Pubkey>,

    /// When the action was submitted
    pub timestamp: i64,
}

impl ActionRecord {
    /// Calculate space needed for the account
    pub fn space(num_signers: usize) -> usize {
        8 + // discriminator
        32 + // action_hash
        1 + // executed
        4 + (num_signers * 32) + // signers vec
        8 // timestamp
    }
}

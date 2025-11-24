use anchor_lang::prelude::*;

#[event]
pub struct MintEvent {
    /// Recipient of the minted sZEC
    pub recipient: Pubkey,

    /// Amount minted
    pub amount: u64,

    /// Nonce at time of minting
    pub nonce: u64,

    /// Timestamp of the mint
    pub timestamp: i64,
}

#[event]
pub struct BurnEvent {
    /// Sender who burned their sZEC
    pub sender: Pubkey,

    /// Amount burned
    pub amount: u64,

    /// Zcash recipient address
    pub zec_recipient: String,

    /// Timestamp of the burn
    pub timestamp: i64,
}

#[event]
pub struct ValidatorSetUpdated {
    /// Previous validator set
    pub old_validators: Vec<Pubkey>,

    /// New validator set
    pub new_validators: Vec<Pubkey>,

    /// New threshold
    pub threshold: u8,

    /// Nonce at time of update
    pub nonce: u64,
}

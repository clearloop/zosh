use anchor_lang::prelude::*;

#[event]
pub struct MintEvent {
    /// Mints to be minted
    pub mints: Vec<(Pubkey, u64)>,

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

//! The header structure of zyphers

/// The header structure of zyphers
pub struct Header {
    /// The height of the block
    pub height: u32,

    /// The parent block hash
    pub parent: [u8; 32],

    /// The merkle root of the state
    pub state: [u8; 32],

    /// The unix timestamp of the block
    pub timestamp: u64,

    /// The merkle root of the transactions
    pub transaction: [u8; 32],
}

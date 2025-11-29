//! The block structure of zorch

use crate::Transaction;

/// The block structure of zorch
pub struct Block {
    /// The header of the block
    pub header: Header,

    /// The transactions of the block
    pub transactions: Vec<Transaction>,
}

/// The header structure of zorch
pub struct Header {
    /// The height of the block
    pub height: u32,

    /// The parent block hash
    pub parent: [u8; 32],

    /// The merkle root of the state
    pub state: [u8; 32],

    /// The merkle root of the transactions
    pub transaction: [u8; 32],
}

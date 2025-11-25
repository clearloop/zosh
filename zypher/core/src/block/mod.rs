//! The block structure of zyphers

use crate::Transaction;
pub use header::Header;

mod header;

/// The block structure of zyphers
pub struct Block {
    /// The header of the block
    pub header: Header,

    /// The transactions of the block
    pub transactions: Vec<Transaction>,
}

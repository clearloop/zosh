//! The storage of zosh

use anyhow::Result;
use zcore::{Block, State};

/// The storage for the zosh bridge
pub trait Storage {
    /// Batch the zosh state
    fn state(&self) -> State;

    /// Batch the operations to the storage
    fn batch(&self, operations: Vec<Operation>) -> Result<()>;

    /// Set the block to the storage
    fn set_block(&self, block: Block) -> Result<()>;

    /// Get the root of the state
    fn root(&self) -> [u8; 32];
}

/// The operation of the storage
pub enum Operation {
    /// Set the value of the key
    Set([u8; 31], Vec<u8>),

    /// Remove the value of the key
    Remove([u8; 31]),
}

//! The storage of zosh

use anyhow::Result;
use zcore::State;

/// The storage for the zosh bridge
pub trait Storage {
    /// Batch the zosh state
    fn state(&self) -> State;

    fn batch(&self, operations: Vec<Operation>) -> Result<()>;
}

/// The operation of the storage
pub enum Operation {
    /// Set the value of the key
    Set([u8; 31], Vec<u8>),

    /// Remove the value of the key
    Remove([u8; 31]),
}

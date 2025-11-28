//! The transaction structure of zorch

use serde::{Deserialize, Serialize};
pub use {
    bridge::{Bridge, Receipt, Refund},
    sync::Sync,
};

mod bridge;
mod sync;

/// The transactions inside of a block
pub struct Transaction {
    /// The bridges of the transaction
    pub bridge: Vec<Bridge>,

    /// The sync of the transaction
    pub sync: Option<Sync>,
}

/// The supported chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Chain {
    /// Solana chain
    Solana,

    /// Zcash chain
    Zcash,
}

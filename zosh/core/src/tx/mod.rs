//! The transaction structure of zorch

pub use bridge::{Bridge, Receipt};
use serde::{Deserialize, Serialize};

mod bridge;

/// The transaction structure of zorch
pub struct Transaction {
    /// The quotes of the transaction
    pub bridge: Vec<Bridge>,
}

/// The supported chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Chain {
    /// Solana chain
    Solana,

    /// Zcash chain
    Zcash,
}

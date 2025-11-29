//! The registry of the supported chains

use serde::{Deserialize, Serialize};

/// The supported chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Chain {
    /// Solana chain
    Solana,

    /// Zcash chain
    Zcash,
}

/// The supported tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Coin {
    /// Zcash coin
    Zec,
}

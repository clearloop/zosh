//! The registry of the supported chains

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// The supported chains
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub enum Chain {
    /// Solana chain
    Solana,

    /// Zcash chain
    Zcash,
}

impl Chain {
    /// Get the bundle limit of the chain
    pub fn max_bundle_size(&self) -> usize {
        match self {
            Chain::Solana => 10,
            Chain::Zcash => 1,
        }
    }
}

/// The supported tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Coin {
    /// Zcash coin
    Zec,
}

impl Display for Coin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Coin::Zec => write!(f, "ZEC"),
        }
    }
}

//! Core types for the zorch network

use serde::{Deserialize, Serialize};
pub use {
    block::{Block, Head, Header},
    ex::Extrinsic,
    state::State,
    util::{Message, ToSig},
};

pub mod bft;
mod block;
pub mod ex;
pub mod req;
pub mod state;
pub mod util;

/// The length of an epoch
///
/// Epoch length for rotating the validators, need
/// to be determined once we can calculate our tps.
pub const EPOCH_LENGTH: usize = 12;

/// The signature type for the Ed25519 algorithm
pub type Ed25519Signature = [u8; 64];

/// The hash type for the zosh network
pub type Hash = [u8; 32];

/// The key type for the trie db
pub type TrieKey = [u8; 31];

/// The supported chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Chain {
    /// Solana chain
    Solana,

    /// Zcash chain
    Zcash,
}

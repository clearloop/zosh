//! Core types for the zorch network

use serde::{Deserialize, Serialize};
pub use {
    block::{Block, Header},
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
pub const EPOCH_LENGTH: u32 = 12;

/// The signature type for the Ed25519 algorithm
pub type Ed25519Signature = [u8; 64];

/// The supported chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Chain {
    /// Solana chain
    Solana,

    /// Zcash chain
    Zcash,
}

/// Extrinsic with sources
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sourced<T> {
    /// The extrinsic
    pub extrinsic: T,

    /// The sources of the extrinsic
    pub sources: Vec<Vec<u8>>,
}

//! Core types for the zorch network

use serde::{Deserialize, Serialize};
pub use {
    block::{Block, Header},
    ex::Extrinsic,
    state::State,
};

pub mod bft;
mod block;
pub mod ex;
pub mod req;
pub mod state;

/// The supported chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Chain {
    /// Solana chain
    Solana,

    /// Zcash chain
    Zcash,
}

/// The message trait
pub trait Message {
    /// Get the message need to sign for the transaction
    fn message(&self) -> Vec<u8>;
}

/// Extrinsic with sources
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sourced<T> {
    /// The extrinsic
    pub extrinsic: T,

    /// The sources of the extrinsic
    pub sources: Vec<Vec<u8>>,
}

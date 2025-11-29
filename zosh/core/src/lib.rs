//! Core types for the zorch network

use serde::{Deserialize, Serialize};
pub use {
    block::{Block, Header},
    state::State,
    ex::Extrinsic,
};

mod block;
pub mod req;
mod state;
pub mod ex;

/// The supported chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Chain {
    /// Solana chain
    Solana,

    /// Zcash chain
    Zcash,
}

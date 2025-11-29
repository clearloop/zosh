//! Core types for the zorch network

use serde::{Deserialize, Serialize};
pub use {
    block::{Block, Header},
    ex::Extrinsic,
    state::State,
};

mod block;
pub mod ex;
pub mod req;
mod state;

/// The supported chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Chain {
    /// Solana chain
    Solana,

    /// Zcash chain
    Zcash,
}

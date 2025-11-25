//! Core types for the zorch network

pub use {
    block::{Block, Header},
    tx::Transaction,
};

mod block;
mod tx;

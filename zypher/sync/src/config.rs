//! Configuration for the sync library

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Configuration for the sync library
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// solana RPC address
    pub solana: SocketAddr,

    /// zcash RPC address
    pub zcash: SocketAddr,
}

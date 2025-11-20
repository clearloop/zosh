//! Configuration for the bridge

use std::net::SocketAddr;

/// Configuration for the bridge
pub struct Config {
    /// The RPC address of the Zcash node
    pub rpc: SocketAddr,

    /// Seed in hex format
    pub seed: String,
}

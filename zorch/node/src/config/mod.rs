//! Configuration for the zorch node

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
pub use {key::Key, network::Network, rpc::Rpc, sync::zcash};

mod key;
mod network;
mod rpc;

/// Configuration for the zorch node
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// sync configurations
    pub rpc: Rpc,

    /// Key configurations
    pub key: Key,

    /// Network
    pub network: Network,
}

impl Config {
    /// Load the configuration from a file
    pub fn load(path: &Path) -> Result<Self> {
        let file = fs::read_to_string(path)?;
        Ok(toml::from_str(&file)?)
    }

    /// Get the zcash light client configuration
    pub fn zcash(&self, cache: &Path) -> zcash::light::Config {
        zcash::light::Config {
            cache: cache.join("chain.db"),
            wallet: cache.join("wallet.db"),
            lightwalletd: self.rpc.lightwalletd.clone(),
            network: self.network.clone().into(),
        }
    }
}

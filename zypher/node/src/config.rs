//! Configuration for the zyphers node

use anyhow::Result;
pub use runtime::config::Key;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use url::Url;

/// Configuration for the zyphers node
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// sync configurations
    pub sync: Rpc,

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
}

/// RPC configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Rpc {
    /// solana RPC address
    pub solana: Url,

    /// zcash RPC address
    pub lightwalletd: Url,
}

/// Network type wrapper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl From<zcash::Network> for Network {
    fn from(network: zcash::Network) -> Self {
        match network {
            zcash::Network::MainNetwork => Network::Mainnet,
            zcash::Network::TestNetwork => Network::Testnet,
        }
    }
}

impl From<Network> for zcash::Network {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => zcash::Network::MainNetwork,
            Network::Testnet => zcash::Network::TestNetwork,
        }
    }
}

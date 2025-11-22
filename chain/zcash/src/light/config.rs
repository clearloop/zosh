//! Zcash light client configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;
use zcash_protocol::consensus;

/// Zcash light client configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Cache directory
    pub cache: PathBuf,

    /// Wallet directory
    pub wallet: PathBuf,

    /// Lightwalletd URL
    pub lightwalletd: Url,

    /// Network
    pub network: Network,
}

/// Network type wrapper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl From<consensus::Network> for Network {
    fn from(network: consensus::Network) -> Self {
        match network {
            consensus::Network::MainNetwork => Network::Mainnet,
            consensus::Network::TestNetwork => Network::Testnet,
        }
    }
}

impl From<Network> for consensus::Network {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => consensus::Network::MainNetwork,
            Network::Testnet => consensus::Network::TestNetwork,
        }
    }
}

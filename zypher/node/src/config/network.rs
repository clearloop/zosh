//! Network configuration

use serde::{Deserialize, Serialize};

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

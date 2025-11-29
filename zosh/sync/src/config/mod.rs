//! Configuration for the zorch node

use crate::{solana, zcash::SignerInfo};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
pub use {crate::zcash, key::Key, network::Network, rpc::Rpc};

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
        let config = path.join("config.toml");
        if !config.exists() {
            return Self::generate(path);
        }
        let file = fs::read_to_string(path.join("config.toml"))?;
        Ok(toml::from_str(&file)?)
    }

    /// Get the zcash light client configuration
    pub fn zcash(&self, cache: &Path) -> Result<zcash::Config> {
        let group: zcash::GroupSigners =
            postcard::from_bytes(&bs58::decode(self.key.zcash.as_str()).into_vec()?)?;
        let ufvk = group.ufvk()?;
        Ok(zcash::Config {
            cache: cache.join("chain.db"),
            wallet: cache.join("wallet.db"),
            lightwalletd: self.rpc.lightwalletd.clone(),
            network: self.network.clone().into(),
            ufvk,
        })
    }

    /// Generate a default configuration file
    pub fn generate(target: &Path) -> Result<Self> {
        let config = Config {
            rpc: Rpc {
                solana: "https://api.mainnet-beta.solana.com".parse()?,
                solana_ws: "wss://api.mainnet-beta.solana.com".parse()?,
                lightwalletd: "http://127.0.0.1:9067".parse()?,
            },
            key: Key {
                zcash: bs58::encode(postcard::to_allocvec(&zcash::GroupSigners::new(3, 2)?)?)
                    .into_string(),
                solana: bs58::encode(postcard::to_allocvec(&solana::GroupSigners::new(3, 2)?)?)
                    .into_string(),
            },
            network: Network::Testnet,
        };
        fs::write(target, toml::to_string_pretty(&config)?)?;
        Ok(config)
    }
}

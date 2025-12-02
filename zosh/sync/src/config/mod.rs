//! Configuration for the zorch node

use crate::{solana, zcash::SignerInfo};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};
pub use {crate::zcash, key::Key, network::Network, rpc::Rpc};

mod key;
mod network;
mod rpc;

/// Environment variable for cache directory override
const ENV_CACHE_DIR: &str = "ZOSH_CACHE_DIR";

/// Environment variable for config directory override
const ENV_CONFIG_DIR: &str = "ZOSH_CONFIG_DIR";

/// Environment variable for Solana RPC endpoint override
const ENV_RPC_SOLANA: &str = "ZOSH_RPC_SOLANA";

/// Environment variable for Solana WebSocket endpoint override
const ENV_RPC_SOLANA_WS: &str = "ZOSH_RPC_SOLANA_WS";

/// Environment variable for Zcash lightwalletd endpoint override
const ENV_RPC_LIGHTWALLETD: &str = "ZOSH_RPC_LIGHTWALLETD";

/// The cache directory
///
/// Can be overridden via `ZOSH_CACHE_DIR` environment variable
pub static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var(ENV_CACHE_DIR)
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .expect("home directory not found")
                .join(".cache")
                .join("zosh")
        })
});

/// The configuration directory
///
/// Can be overridden via `ZOSH_CONFIG_DIR` environment variable
pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var(ENV_CONFIG_DIR)
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .expect("home directory not found")
                .join(".config")
                .join("zosh")
        })
});

/// The configuration file
pub static CONFIG_FILE: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("config.toml"));

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
    ///
    /// Environment variables can override config file values:
    /// - `ZOSH_RPC_SOLANA` - Solana RPC endpoint
    /// - `ZOSH_RPC_SOLANA_WS` - Solana WebSocket endpoint
    /// - `ZOSH_RPC_LIGHTWALLETD` - Zcash lightwalletd endpoint
    pub fn load() -> Result<Self> {
        let mut config = if !CONFIG_FILE.exists() {
            Self::generate(CONFIG_FILE.as_path())?
        } else {
            let file = fs::read_to_string(CONFIG_FILE.as_path())?;
            toml::from_str(&file)?
        };

        // Apply environment variable overrides for RPC endpoints
        if let Ok(solana) = env::var(ENV_RPC_SOLANA) {
            config.rpc.solana = solana.parse()?;
        }
        if let Ok(solana_ws) = env::var(ENV_RPC_SOLANA_WS) {
            config.rpc.solana_ws = solana_ws.parse()?;
        }
        if let Ok(lightwalletd) = env::var(ENV_RPC_LIGHTWALLETD) {
            config.rpc.lightwalletd = lightwalletd.parse()?;
        }

        Ok(config)
    }

    /// Get the zcash light client configuration
    pub fn zcash(&self) -> Result<zcash::Config> {
        let group: zcash::GroupSigners =
            postcard::from_bytes(&bs58::decode(self.key.zcash.as_str()).into_vec()?)?;
        let ufvk = group.ufvk()?;
        Ok(zcash::Config {
            cache: CACHE_DIR.join("chain.db"),
            wallet: CACHE_DIR.join("wallet.db"),
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

//! sync library for ZorchBridge

use anyhow::Result;
use solana_sdk::signature::Keypair;
use std::path::Path;
pub use {config::Config, event::Event, solana::ZoshClient, zcash::Light};

pub mod config;
mod dev;
mod event;
pub mod solana;
pub mod zcash;

/// The sync data source
pub struct Sync {
    /// The zcash light client
    pub zcash: Light,

    /// The solana client
    pub solana: ZoshClient,
}

impl Sync {
    /// Create a new sync instance
    pub async fn new(cache: &Path, config: &Config) -> Result<Self> {
        let zcash = config.zcash(cache)?;
        let keypair = Keypair::from_base58_string(&config.key.solana);
        let solana = ZoshClient::new(
            config.rpc.solana.to_string(),
            config.rpc.solana_ws.to_string(),
            keypair,
        )?;
        let zcash = Light::new(&zcash).await?;
        Ok(Self { zcash, solana })
    }
}

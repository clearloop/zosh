//! sync library for ZorchBridge

use crate::solana::SolanaClient;
use anyhow::Result;
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
    pub solana: SolanaClient,
}

impl Sync {
    /// Create a new sync instance
    pub async fn new(cache: &Path, config: &Config) -> Result<Self> {
        let zcash = config.zcash(cache)?;
        let zcash = Light::new(&zcash).await?;
        let solana = SolanaClient::new(config).await?;
        Ok(Self { zcash, solana })
    }
}

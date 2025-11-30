//! sync library for ZorchBridge

use crate::solana::SolanaClient;
use anyhow::Result;
use tokio::sync::mpsc;
use zcore::ex::Bridge;
pub use {config::Config, solana::ZoshClient, zcash::Light};

pub mod config;
pub mod solana;
mod validate;
pub mod zcash;

/// The sync data source
pub struct Sync {
    /// The zcash light client
    pub zcash: Light,

    /// The solana client
    pub solana: SolanaClient,
}

impl Sync {
    /// Load the sync configuration
    pub async fn load() -> Result<Self> {
        let config = Config::load()?;
        Self::new(&config).await
    }

    /// Create a new sync instance
    pub async fn new(config: &Config) -> Result<Self> {
        let zconf = config.zcash()?;
        let zcash = Light::new(&zconf).await?;
        let solana = SolanaClient::new(config).await?;
        Ok(Self { zcash, solana })
    }

    /// Start the sync
    pub async fn start(&mut self, tx: mpsc::Sender<Bridge>) -> Result<()> {
        tokio::select! {
            r = self.zcash.subscribe(tx.clone()) => r,
            r = self.solana.subscribe(tx.clone()) => r
        }
    }
}

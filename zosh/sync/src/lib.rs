//! sync library for ZorchBridge

use crate::solana::SolanaClient;
use anyhow::Result;
use tokio::sync::mpsc;
pub use {config::Config, event::Event, solana::ZoshClient, zcash::Light};

pub mod config;
mod event;
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
    /// Create a new sync instance
    pub async fn new(config: &Config) -> Result<Self> {
        let zconf = config.zcash()?;
        let zcash = Light::new(&zconf).await?;
        let solana = SolanaClient::new(config).await?;
        Ok(Self { zcash, solana })
    }

    /// Start the sync
    pub async fn start(&mut self, tx: mpsc::Sender<Event>) -> Result<()> {
        let mut zsync = self.zcash.duplicate().await?;
        tokio::select! {
            r = zsync.sync_forever() => r,
            r = self.zcash.subscribe(tx.clone()) => r,
            r = self.solana.subscribe(tx.clone()) => r
        }
    }
}

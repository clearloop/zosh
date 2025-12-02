//! sync library for ZorchBridge

use crate::solana::SolanaClient;
use anyhow::Result;
use tokio::sync::mpsc;
use zcore::ex::Bridge;
pub use {config::Config, encoder::ChainFormatEncoder, solana::ZoshClient, zcash::ZcashClient};

mod bundle;
pub mod config;
mod encoder;
pub mod solana;
pub mod zcash;

/// The sync data source
pub struct Sync {
    /// The development MPC
    pub dev_zcash_mpc: zcash::GroupSigners,

    /// The development MPC
    pub dev_solana_mpc: solana::GroupSigners,

    /// The solana client
    pub solana: SolanaClient,

    /// The zcash light client
    pub zcash: ZcashClient,

    /// unresolved bundles
    pub unresolved: Vec<Bridge>,
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
        let zcash = ZcashClient::new(&zconf).await?;
        let solana = SolanaClient::new(config).await?;
        let dev_zcash_mpc: zcash::GroupSigners =
            postcard::from_bytes(&bs58::decode(&config.key.zcash).into_vec()?)?;
        let dev_solana_mpc: solana::GroupSigners =
            postcard::from_bytes(&bs58::decode(&config.key.solana).into_vec()?)?;
        Ok(Self {
            dev_solana_mpc,
            dev_zcash_mpc,
            zcash,
            solana,
            unresolved: Default::default(),
        })
    }

    /// Spawn the sync service
    pub fn spawn(self, tx: mpsc::Sender<Bridge>) {
        tokio::spawn(async move { self.start(tx).await });
    }

    /// Start the sync
    pub async fn start(mut self, tx: mpsc::Sender<Bridge>) {
        tokio::select! {
            r = self.zcash.subscribe(tx.clone()) => r,
            r = self.solana.subscribe(tx.clone()) => r
        }
    }
}

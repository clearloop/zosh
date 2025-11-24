//! Zcash tools

use crate::config::Network;
use anyhow::Result;
use clap::Parser;
use std::path::Path;
use url::Url;
use zcash::light::{Config, Light};
/// Zcash tools
#[derive(Parser)]
pub enum Zcash {
    // /// Import a unified full viewing key
    // Import {
    //     /// The birth of the account
    //     #[clap(short, long)]
    //     birth: u32,
    //
    //     /// The name of the account
    //     #[clap(short, long)]
    //     name: String,
    //
    //     /// The unified full viewing key to import
    //     #[clap(short, long)]
    //     ufvk: String,
    // },
    /// Prints the remote light client status
    Light { url: Url },

    /// Syncs the local wallet with the remote light client
    Sync { url: Url },
}

impl Zcash {
    /// Run the zcash command
    pub async fn run(&self, cache: &Path, network: Network) -> Result<()> {
        match self {
            Self::Light { url } => self.light(cache, url, network).await,
            Self::Sync { url } => self.sync(cache, url, network).await,
        }
    }

    /// Get the light client info
    async fn light(&self, cache: &Path, url: &Url, network: Network) -> Result<()> {
        let config = Config {
            cache: cache.join("chain.db"),
            wallet: cache.join("wallet.db"),
            lightwalletd: url.clone(),
        };
        let mut light = Light::new(&config, network.into()).await?;
        light.info().await?;
        Ok(())
    }

    /// Sync the local wallet with the remote light client
    async fn sync(&self, cache: &Path, url: &Url, network: Network) -> Result<()> {
        let config = Config {
            cache: cache.join("chain.db"),
            wallet: cache.join("wallet.db"),
            lightwalletd: url.clone(),
        };
        let mut light = Light::new(&config, network.clone().into()).await?;
        light.sync().await?;
        Ok(())
    }
}

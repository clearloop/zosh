//! Zcash tools

use crate::Config;
use anyhow::Result;
use clap::Parser;
use std::path::Path;
use zcash::{
    light::Light,
    signer::{GroupSigners, SignerInfo},
};

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
    Light,

    /// Syncs the local wallet with the remote light client
    Sync,

    /// Prints the signer info
    Info,
}

impl Zcash {
    /// Run the zcash command
    pub async fn run(&self, cache: &Path, config: &Config) -> Result<()> {
        let cfg = config.zcash(cache);
        match self {
            Self::Light => self.light(&cfg).await,
            Self::Sync => self.sync(&cfg).await,
            Self::Info => self.info(&config),
        }
    }

    fn info(&self, cfg: &Config) -> Result<()> {
        let network: zcash::Network = cfg.network.clone().into();
        let bytes = bs58::decode(&cfg.key.zcash).into_vec()?;
        let group: GroupSigners = postcard::from_bytes(&bytes)?;
        println!(
            "Unifed orchard address: {}",
            group.unified_address()?.encode(&network)
        );

        println!(
            "Unified full viewing key: {}",
            group.ufvk()?.encode(&network)
        );
        Ok(())
    }

    /// Get the light client info
    async fn light(&self, cfg: &zcash::light::Config) -> Result<()> {
        let mut light = Light::new(&cfg).await?;
        light.info().await?;
        Ok(())
    }

    /// Sync the local wallet with the remote light client
    async fn sync(&self, cfg: &zcash::light::Config) -> Result<()> {
        let mut light = Light::new(&cfg).await?;
        light.sync().await?;
        Ok(())
    }
}

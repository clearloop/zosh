//! Zcash tools

use crate::config::Config;
use crate::zcash::{
    self,
    light::Light,
    signer::{GroupSigners, SignerInfo},
    AddressCodec, UnifiedAddress, UnifiedFullViewingKey,
};
use anyhow::Result;
use clap::Parser;
use std::path::Path;
use zcash_client_backend::data_api::wallet::ConfirmationsPolicy;
use zcash_client_backend::data_api::WalletRead;
use zcash_client_backend::proto::service::Empty;

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

    /// Get the wallet summary
    Summary,

    /// Import a unified full viewing key
    Import {
        /// The unified full viewing key to import
        ufvk: String,

        /// The name of the account
        name: String,
    },

    Send {
        /// The recipient address
        #[clap(short, long)]
        recipient: String,

        /// The amount to send
        #[clap(short, long)]
        amount: f32,
    },
}

impl Zcash {
    /// Run the zcash command
    pub async fn run(&self, cache: &Path, config: &Config) -> Result<()> {
        let cfg = config.zcash(cache);
        match self {
            Self::Light => self.light(&cfg).await,
            Self::Sync => self.sync(&cfg).await,
            Self::Info => self.info(config),
            Self::Summary => self.summary(&cfg).await,
            Self::Import { ufvk, name } => self.import(&cfg, ufvk, name).await,
            Self::Send { recipient, amount } => {
                self.send(&cfg, &config.key.zcash, recipient, *amount).await
            }
        }
    }

    fn info(&self, cfg: &Config) -> Result<()> {
        let network: zcash::Network = cfg.network.clone().into();
        let bytes = bs58::decode(&cfg.key.zcash).into_vec()?;
        let group: GroupSigners = postcard::from_bytes(&bytes)?;
        println!(
            "Unified orchard address: {}",
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
        let mut light = Light::new(cfg).await?;
        let info = light.client.get_lightd_info(Empty {}).await?;
        println!("Light client info: {:?}", info);
        Ok(())
    }

    /// Sync the local wallet with the remote light client
    async fn sync(&self, cfg: &zcash::light::Config) -> Result<()> {
        let mut light = Light::new(cfg).await?;
        light.sync().await?;
        Ok(())
    }

    /// Import a unified full viewing key
    async fn import(&self, cfg: &zcash::light::Config, ufvk: &str, name: &str) -> Result<()> {
        let mut light = Light::new(cfg).await?;
        light
            .import(
                name,
                UnifiedFullViewingKey::decode(&light.network, ufvk)
                    .map_err(|e| anyhow::anyhow!(e))?,
            )
            .await?;
        Ok(())
    }

    /// Get the wallet summary
    async fn summary(&self, cfg: &zcash::light::Config) -> Result<()> {
        let light = Light::new(cfg).await?;
        let summary = light
            .wallet
            .get_wallet_summary(ConfirmationsPolicy::new_symmetrical(1.try_into().unwrap()))?
            .ok_or(anyhow::anyhow!("Failed to get wallet summary"))?;

        println!("Wallet summary: {:?}", summary);
        Ok(())
    }

    /// Send a fund to a orchard address
    async fn send(
        &self,
        cfg: &zcash::light::Config,
        group: &str,
        recipient: &str,
        amount: f32,
    ) -> Result<()> {
        let mut light = Light::new(cfg).await?;
        light
            .send(
                postcard::from_bytes(&bs58::decode(group).into_vec()?)?,
                UnifiedAddress::decode(&light.network, recipient)
                    .map_err(|e| anyhow::anyhow!(e))?,
                amount,
            )
            .await?;

        Ok(())
    }
}

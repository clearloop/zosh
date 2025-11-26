//! Solana command line interface

use crate::Config;
use anyhow::Result;
use clap::Parser;
use solana_sdk::signature::Keypair;
use zorch::client::ZorchClient;

/// Solana command line interface
#[derive(Parser)]
pub enum Solana {
    /// Get the current bridge state
    State,

    /// Get or update the current metadata
    Metadata {
        /// The new of our bridged ZEC
        #[clap(short, long, default_value = "Zorch ZEC")]
        name: String,
        /// The new symbol of our bridged ZEC
        #[clap(short, long, default_value = "zrcZEC")]
        symbol: String,
        /// The new URI of our bridged ZEC
        #[clap(
            long,
            default_value = "https://obamyvsl2qpmlutsi2smszmcijjsog7euvnfl4quigts5ie2tlua.arweave.net/cEDMVkvUHsXSckakyWWCQlMnG-SlWlXyFEGnLqCamug"
        )]
        uri: String,
        /// Whether to update the metadata
        #[clap(short, long, default_value = "false")]
        update: bool,
    },
}

impl Solana {
    /// Run the Solana command
    pub async fn run(&self, config: &Config) -> Result<()> {
        let keypair = Keypair::from_base58_string(&config.key.solana);
        let client = ZorchClient::new(
            config.rpc.solana.to_string(),
            config.rpc.solana_ws.to_string(),
            keypair,
        )?;

        match self {
            Self::State => self.initialize(client).await,
            Self::Metadata {
                name,
                symbol,
                uri,
                update,
            } => self.metadata(client, name, symbol, uri, *update).await,
        }
    }

    /// Initialize the bridge
    async fn initialize(&self, client: ZorchClient) -> Result<()> {
        if let Ok(state) = client.bridge_state().await {
            println!("{state:#?}");
            return Ok(());
        }

        let _ = client.initialize(vec![client.payer()], 1).await?;
        let state = client.bridge_state().await?;
        println!("{state:#?}");
        Ok(())
    }

    /// Get the current metadata
    async fn metadata(
        &self,
        client: ZorchClient,
        name: &str,
        symbol: &str,
        uri: &str,
        update: bool,
    ) -> Result<()> {
        if let Ok(metadata) = client.metadata().await {
            if !update {
                println!("{metadata:#?}");
                return Ok(());
            }
        }

        client
            .update_metadata(name.to_owned(), symbol.to_owned(), uri.to_owned())
            .await?;
        let metadata = client.metadata().await?;
        println!("{metadata:#?}");
        Ok(())
    }
}

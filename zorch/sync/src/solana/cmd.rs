//! Solana command line interface

use std::str::FromStr;

use crate::Config;
use anyhow::Result;
use clap::Parser;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use zorch::client::ZorchClient;

/// Solana command line interface
#[derive(Parser)]
pub enum Solana {
    /// Get the current bridge state
    State,

    /// Get the current zec balance for a recipient
    Balance {
        /// The address of the recipient
        #[clap(short, long)]
        address: Option<String>,
    },

    /// Burn sZEC to bridge back to Zcash
    Burn {
        /// The amount to burn
        #[clap(short, long)]
        amount: u64,

        /// The address of the recipient
        #[clap(short, long)]
        recipient: String,
    },

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
            Self::Balance { address } => self.balance(client, address.clone()).await,
            Self::Burn { amount, recipient } => self.burn(client, *amount, recipient.clone()).await,
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

    /// Burn sZEC to bridge back to Zcash
    async fn burn(&self, client: ZorchClient, amount: u64, address: String) -> Result<()> {
        client.send_burn(amount, address).await?;
        let balance = client.zec_balance(client.payer()).await?;
        println!("{balance:#?}");
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

    /// Get the current zec balance for a recipient
    async fn balance(&self, client: ZorchClient, address: Option<String>) -> Result<()> {
        let address = address.unwrap_or_else(|| client.payer().to_string());
        let recipient = Pubkey::from_str(&address)?;
        let balance = client.zec_balance(recipient).await?;
        println!("{balance:#?}");
        Ok(())
    }
}

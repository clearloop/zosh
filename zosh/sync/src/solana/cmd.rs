//! Solana command line interface

use crate::{
    solana::{GroupSigners, SolanaClient, SolanaSignerInfo},
    Config,
};
use anyhow::Result;
use clap::Parser;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

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

    DevMint {
        /// The address of the recipient
        #[clap(
            short,
            long,
            default_value = "FVyaCqkMQmju9i36gprXpwo5jMU12FLoPvwBMocYdPjt"
        )]
        recipient: Pubkey,

        /// The amount to mint
        #[clap(short, long)]
        amount: u64,
    },

    /// Get or update the current metadata
    Metadata {
        /// The new of our bridged ZEC
        #[clap(short, long, default_value = "Zosh ZEC")]
        name: String,
        /// The new symbol of our bridged ZEC
        #[clap(short, long, default_value = "zoZEC")]
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
        let mpc: GroupSigners =
            postcard::from_bytes(&bs58::decode(&config.key.solana).into_vec()?)?;
        let client = SolanaClient::new(config).await?;

        match self {
            Self::State => self.initialize(client, mpc.pubkey()).await,
            Self::Balance { address } => self.balance(client, address.clone()).await,
            Self::Burn { amount, recipient } => self.burn(client, *amount, recipient.clone()).await,
            Self::DevMint { recipient, amount } => {
                self.dev_mint(client, mpc, *recipient, *amount).await
            }
            Self::Metadata {
                name,
                symbol,
                uri,
                update,
            } => self.metadata(client, name, symbol, uri, *update).await,
        }
    }

    async fn dev_mint(
        &self,
        client: SolanaClient,
        dev_mpc: GroupSigners,
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
        let mints = vec![zosh::types::MintEntry { recipient, amount }];
        let tx = client.mint(mints).await?;
        let _signature = client.dev_sign_and_send(tx, &dev_mpc).await?;
        self.balance(client, Some(recipient.to_string())).await?;
        Ok(())
    }

    /// Initialize the bridge
    async fn initialize(&self, client: SolanaClient, mpc: Pubkey) -> Result<()> {
        if let Ok(state) = client.bridge_state().await {
            println!("{state:#?}");
            return Ok(());
        }

        // prints the state
        client.initialize(mpc).await?;
        let state = client.bridge_state().await?;
        println!("{state:#?}");
        Ok(())
    }

    /// Burn sZEC to bridge back to Zcash
    async fn burn(&self, client: SolanaClient, amount: u64, address: String) -> Result<()> {
        client.burn(amount, address).await?;

        // prints the balance
        let balance = client.zec_balance(client.program.payer()).await?;
        println!("{balance:#?}");
        Ok(())
    }

    /// Get the current metadata
    async fn metadata(
        &self,
        client: SolanaClient,
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

        // prints the metadata
        let metadata = client.metadata().await?;
        println!("{metadata:#?}");
        Ok(())
    }

    /// Get the current zec balance for a recipient
    async fn balance(&self, client: SolanaClient, address: Option<String>) -> Result<()> {
        let address = address.unwrap_or_else(|| client.program.payer().to_string());
        let recipient = Pubkey::from_str(&address)?;
        let balance = client.zec_balance(recipient).await?;
        println!("{balance:#?}");
        Ok(())
    }
}

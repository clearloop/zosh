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
}

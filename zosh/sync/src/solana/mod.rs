//! Solana sync library

use crate::Config;
use anyhow::Result;
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
pub use solana_sdk::{pubkey::Pubkey, signer::keypair::Keypair};
use std::ops::Deref;
pub use zosh::client::ZoshClient;
pub use {cmd::Solana, signer::GroupSigners};

mod cmd;
mod signer;
mod sub;

/// Solana client
pub struct SolanaClient {
    /// The transaction client
    pub tx: ZoshClient,

    /// The subscription client
    pub sub: PubsubClient,
}

impl SolanaClient {
    /// Create a new solana client
    pub async fn new(config: &Config) -> Result<Self> {
        let keypair = Keypair::from_base58_string(&config.key.solana);
        let solana = ZoshClient::new(
            config.rpc.solana.to_string(),
            config.rpc.solana_ws.to_string(),
            keypair,
        )?;

        let sub = PubsubClient::new(config.rpc.solana_ws.as_ref()).await?;
        Ok(Self { tx: solana, sub })
    }
}

impl Deref for SolanaClient {
    type Target = ZoshClient;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

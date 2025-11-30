//! Solana sync library

use crate::{ChainFormatEncoder, Config};
use anyhow::Result;
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
pub use solana_sdk::{
    pubkey::Pubkey, signature::Signature, signer::keypair::Keypair, transaction::Transaction,
};

use zcore::{
    ex::{Bridge, BridgeBundle},
    registry::Chain,
};
pub use zosh::client::ZoshClient;
use zosh::types::MintEntry;
pub use {
    cmd::Solana,
    signer::{GroupSigners, SolanaSignerInfo},
};

mod cmd;
pub mod dev;
mod signer;
mod sub;

/// Solana client
pub struct SolanaClient {
    /// The transaction client
    pub tx: ZoshClient,

    /// The subscription client
    pub sub: PubsubClient,

    /// The development MPC
    ///
    /// This is used to sign transactions for development purposes.
    pub dev_mpc: GroupSigners,
}

impl SolanaClient {
    /// Create a new solana client
    pub async fn new(config: &Config) -> Result<Self> {
        let authority = dev::load_authority()?;
        let dev_mpc: GroupSigners =
            postcard::from_bytes(&bs58::decode(&config.key.solana).into_vec()?)?;
        let solana = ZoshClient::new(
            config.rpc.solana.to_string(),
            config.rpc.solana_ws.to_string(),
            authority,
        )?;

        let sub = PubsubClient::new(config.rpc.solana_ws.as_ref()).await?;
        Ok(Self {
            tx: solana,
            sub,
            dev_mpc,
        })
    }

    /// Bundle bridge transactions
    ///
    /// IMPORTANT: things should be checked in the outer layer:
    /// - the number of bridges is not too many
    /// - the target chain is valid
    /// - the source chain tx is valid
    pub async fn bundle(&self, bridges: &[Bridge]) -> Result<(BridgeBundle, Transaction)> {
        let mut bundle = BridgeBundle::new(Chain::Solana);
        let mut mints = Vec::new();

        // Check if the target chain is valid
        for bridge in bridges {
            let recipient = bridge.recipient.solana_address()?;
            mints.push(MintEntry {
                recipient,
                amount: bridge.amount,
            });
        }

        let transaction = self.mint(mints).await?;
        let blockhash = transaction.message.recent_blockhash;
        bundle.data = blockhash.to_bytes().to_vec();
        Ok((bundle, transaction))
    }
}

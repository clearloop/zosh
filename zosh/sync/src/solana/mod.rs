//! Solana sync library

use crate::Config;
use anyhow::Result;
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
pub use solana_sdk::{pubkey::Pubkey, signer::keypair::Keypair};
use solana_sdk::{signature::Signature, signer::EncodableKey, transaction::Transaction};
use std::ops::Deref;
use zcore::ToSig;
pub use zosh::client::ZoshClient;
pub use {
    cmd::Solana,
    signer::{GroupSigners, SolanaSignerInfo},
};

mod cmd;
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
        let authority = load_authority()?;
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

    /// Mint tokens for development purposes
    pub async fn dev_mint(
        &self,
        recipient: Pubkey,
        amount: u64,
        mpc: &GroupSigners,
    ) -> Result<Signature> {
        let mints = vec![zosh::types::MintEntry { recipient, amount }];
        let tx = self.tx.mint(mints).await?;
        let signature = self.dev_sign_and_send(tx, &mpc).await?;
        Ok(signature)
    }

    /// Update the MPC for development purposes
    pub async fn dev_update_mpc(&self, mpc: &GroupSigners) -> Result<Signature> {
        let tx = self.tx.update_mpc(mpc.pubkey()).await?;
        let signature = self.dev_sign_and_send(tx, mpc).await?;
        Ok(signature)
    }

    /// Sign and send a transaction
    pub async fn dev_sign_and_send(
        &self,
        mut tx: Transaction,
        signer: &GroupSigners,
    ) -> Result<Signature> {
        let latest_blockhash = self.tx.latest_blockhash().await?;
        tx.message.recent_blockhash = latest_blockhash;
        let signature = signer.sign(&tx.message_data())?.serialize()?.ed25519()?;
        tx.signatures = vec![signature.into()];
        self.tx.send(tx).await
    }
}

impl Deref for SolanaClient {
    type Target = ZoshClient;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

/// Load the authority keypair from the filesystem
pub fn load_authority() -> Result<Keypair> {
    let home = dirs::home_dir().ok_or(anyhow::anyhow!("Home directory not found"))?;
    let authority = Keypair::read_from_file(home.join(".config/solana/id.json"))
        .map_err(|e| anyhow::anyhow!("Error reading `~/.config/solana/id.json`: {}", e))?;
    Ok(authority)
}

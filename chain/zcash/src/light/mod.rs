//! Zcash light client

use anyhow::Result;
use cache::BlockDb;
pub use config::Config;
use rusqlite::Connection;
use std::path::Path;
use tonic::transport::Channel;
use zcash_client_backend::{
    proto::service::compact_tx_streamer_client::CompactTxStreamerClient, sync,
};
use zcash_client_sqlite::{util::SystemClock, WalletDb};
use zcash_protocol::consensus::Network;

mod cache;
mod config;

/// Zcash light client
pub struct Light {
    /// Block database connection
    pub block: BlockDb,

    /// Wallet database path
    pub wallet: WalletDb<Connection, Network, SystemClock, rand_core::OsRng>,

    /// Compact transaction streamer client
    pub client: CompactTxStreamerClient<Channel>,
}

impl Light {
    /// Create a new light client
    pub async fn new(config: &Config) -> Result<Self> {
        let cache = Path::new(&config.cache);
        let block = BlockDb::for_path(cache)?;

        // create the wallet database
        let wallet = WalletDb::for_path(
            config.wallet.as_path(),
            config.network.clone().into(),
            SystemClock,
            rand_core::OsRng,
        )?;

        // setup the lightwalletd client
        let channel = Channel::from_shared(config.lightwalletd.to_string())?
            .connect()
            .await?;
        let client = CompactTxStreamerClient::new(channel);

        // wrap to the light client
        Ok(Self {
            block,
            wallet,
            client,
        })
    }

    /// Sync the wallet
    pub async fn sync(&mut self) -> Result<()> {
        sync::run(
            &mut self.client,
            &self.wallet.params().clone(),
            &self.block,
            &mut self.wallet,
            100,
        )
        .await
        .map_err(Into::into)
    }
}

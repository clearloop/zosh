//! Zcash light client

use std::path::Path;

use crate::light::cache::CacheDb;
use anyhow::Result;
pub use config::Config;
use rusqlite::Connection;
use tonic::transport::Channel;
use zcash_client_backend::{
    proto::service::compact_tx_streamer_client::CompactTxStreamerClient, sync,
};
use zcash_client_sqlite::{chain, util::SystemClock, BlockDb, WalletDb};
use zcash_protocol::consensus::Network;

mod cache;
mod config;

/// Zcash light client
pub struct Light {
    /// Block database connection
    pub block: CacheDb,

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
        chain::init::init_cache_database(&block)?;
        let block = CacheDb::from(block);

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

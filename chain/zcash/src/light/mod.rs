//! Zcash light client

use anyhow::Result;
use cache::BlockDb;
pub use config::{Config, Network};
use rusqlite::Connection;
use std::path::Path;
use tonic::transport::Channel;
use zcash_client_backend::{
    data_api::{chain::ChainState, AccountBirthday, AccountPurpose, WalletWrite},
    proto::service::{compact_tx_streamer_client::CompactTxStreamerClient, BlockId, Empty},
    sync,
};
use zcash_client_sqlite::{util::SystemClock, WalletDb};
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_protocol::consensus;

mod cache;
mod config;

/// Zcash light client
pub struct Light {
    /// Block database connection
    pub block: BlockDb,

    /// Wallet database path
    pub wallet: WalletDb<Connection, consensus::Network, SystemClock, rand_core::OsRng>,

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

    /// Get the light client info
    pub async fn info(&mut self) -> Result<()> {
        let response = self.client.get_lightd_info(Empty {}).await?;
        let info = response.into_inner();
        println!("Light client info: {:?}", info);
        Ok(())
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

    /// Import a unified full viewing key
    pub async fn import(
        &mut self,
        name: &str,
        birth: u32,
        ufvk: UnifiedFullViewingKey,
    ) -> Result<()> {
        let block = self
            .client
            .get_block(BlockId {
                height: birth as u64,
                hash: Default::default(),
            })
            .await?
            .into_inner();

        let chain_state = ChainState::empty(block.height(), block.hash());
        self.wallet.import_account_ufvk(
            name,
            &ufvk,
            &AccountBirthday::from_parts(chain_state, None),
            AccountPurpose::ViewOnly,
            None,
        )?;
        Ok(())
    }
}

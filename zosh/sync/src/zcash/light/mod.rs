//! Zcash light client

use anyhow::Result;
use cache::BlockDb;
pub use config::Config;
use rusqlite::Connection;
use std::{fs, path::Path};
use tonic::transport::{Channel, ClientTlsConfig};
use zcash_client_backend::{
    data_api::WalletRead, proto::service::compact_tx_streamer_client::CompactTxStreamerClient,
};
use zcash_client_sqlite::{util::SystemClock, wallet, WalletDb};
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_protocol::consensus::Network;

mod api;
mod cache;
mod config;
mod sub;
mod tx;

/// Zcash light client
pub struct ZcashClient {
    /// Block database connection
    pub block: BlockDb,

    /// Wallet database path
    pub wallet: WalletDb<Connection, Network, SystemClock, rand_core::OsRng>,

    /// Compact transaction streamer client
    pub client: CompactTxStreamerClient<Channel>,

    /// The network of the light client
    pub network: Network,

    /// The unified full viewing key of the light client
    pub ufvk: UnifiedFullViewingKey,
}

impl ZcashClient {
    /// Create a new light client
    pub async fn new(config: &Config) -> Result<Self> {
        if let Some(parent) = config.cache.parent() {
            fs::create_dir_all(parent)?;
        }

        let cache = Path::new(&config.cache);
        let block = BlockDb::for_path(cache)?;

        // create the wallet database
        let mut wallet = WalletDb::for_path(
            config.wallet.as_path(),
            config.network,
            SystemClock,
            rand_core::OsRng,
        )?;

        // Initialize the wallet database schema (creates all required tables)
        // The seed parameter is None since we're not using a seed for this wallet
        wallet::init::init_wallet_db(&mut wallet, None)?;
        let tls = ClientTlsConfig::new().with_native_roots();
        let channel = Channel::from_shared(config.lightwalletd.to_string())?
            .tls_config(tls)?
            .connect()
            .await?;
        let client = CompactTxStreamerClient::new(channel);
        let mut this = Self {
            block,
            wallet,
            client,
            network: config.network,
            ufvk: config.ufvk.clone(),
        };

        // import the account if it doesn't exist
        if this.wallet.get_account_for_ufvk(&config.ufvk)?.is_none() {
            this.import("zosh", config.ufvk.clone(), 3710700).await?;
        }

        // wrap to the light client
        Ok(this)
    }
}

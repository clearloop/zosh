//! The development node implementation

use crate::storage::Parity;
use anyhow::Result;
use runtime::{Config, Pool, Runtime, Storage};
use std::{net::SocketAddr, sync::Arc};
use sync::{config::CACHE_DIR, Sync};
use tokio::sync::{broadcast, mpsc, Mutex};
use zcore::ex::Bridge;

mod author;
mod genesis;
mod relay;

/// The development node implementation
pub struct Dev {
    /// The storage of the development node
    pub parity: Arc<Parity>,

    /// The pool of the development node
    pub pool: Arc<Mutex<Pool>>,

    /// The runtime
    pub runtime: Runtime<Development>,

    /// Stats broadcast sender for WebSocket subscriptions
    pub stats_tx: broadcast::Sender<zoshui::db::Stats>,
}

impl Dev {
    /// Create a new development node
    pub async fn new() -> Result<Self> {
        let uidb = zoshui::Database::new(CACHE_DIR.join("ui.db").as_ref())?;
        uidb.init()?;

        // Create broadcast channel for stats updates
        let (stats_tx, _) = broadcast::channel(16);

        let parity = Arc::new(Parity::try_from(CACHE_DIR.join("chain"))?);
        let hook = zoshui::UIHook::new(uidb, stats_tx.clone());
        let runtime = Runtime::new(hook, parity.clone(), 1).await?;
        let pool = runtime.pool.clone();
        if parity.is_empty()? {
            parity.commit(genesis::commit()?)?;
        }
        Ok(Self {
            runtime,
            pool,
            parity,
            stats_tx,
        })
    }

    /// Start the development node
    pub async fn start(self, address: SocketAddr) -> Result<()> {
        tracing::info!("Starting the development node");
        let Dev {
            parity,
            pool,
            runtime,
            stats_tx,
        } = self;
        let sync = Sync::load().await?;
        zoshui::spawn(runtime.hook.db.clone(), address, stats_tx);
        author::spawn(runtime)?;

        // spawn the sync service
        let (tx, rx) = mpsc::channel::<Bridge>(512);
        sync.spawn(tx);
        relay::spawn(parity, pool, rx).await?;
        let _ = tokio::signal::ctrl_c().await;
        Ok(())
    }
}

/// The development node configuration
#[derive(Clone)]
pub struct Development;

impl Config for Development {
    type Hook = zoshui::UIHook;
    type Storage = Arc<Parity>;
}

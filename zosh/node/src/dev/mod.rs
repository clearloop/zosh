//! The development node implementation

use crate::{rpc::Rpc, storage::Parity};
use anyhow::Result;
use hook::DevHook;
use rpc::server::SubscriptionManager;
use runtime::{Config, Pool, Runtime};
use std::{net::SocketAddr, sync::Arc};
use sync::{config::CACHE_DIR, Event, Sync};
use tokio::sync::{mpsc, Mutex};

mod author;
mod hook;
mod relay;

/// The development node implementation
pub struct Dev {
    /// The runtime
    pub runtime: Runtime<Development>,

    /// The pool of the development node
    pub pool: Arc<Mutex<Pool>>,

    /// The RPC service
    pub rpc: Rpc<Parity>,
}

impl Dev {
    /// Create a new development node
    pub async fn new() -> Result<Self> {
        let manager = SubscriptionManager::default();
        let hook = DevHook::new(manager.clone());
        let parity = Arc::new(Parity::try_from(CACHE_DIR.join("chain"))?);
        let runtime = Runtime::new(hook, parity.clone()).await?;
        let pool = runtime.pool.clone();
        let rpc = Rpc::new(parity, manager);
        Ok(Self { runtime, pool, rpc })
    }

    /// Start the development node
    pub async fn start(self, address: SocketAddr) -> Result<()> {
        let Dev { runtime, pool, rpc } = self;
        let mut sync = Sync::load().await?;
        let (tx, rx) = mpsc::channel::<Event>(512);
        tokio::select! {
            r = relay::start(pool, rx) => r,
            r = author::start(runtime) => r,
            r = rpc.start(address) => r,
            r = sync.start(tx) => r,
        }
    }
}

/// The development node configuration
pub struct Development;

impl Config for Development {
    type Hook = DevHook;
    type Storage = Arc<Parity>;
}

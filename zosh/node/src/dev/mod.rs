//! The development node implementation

use crate::{rpc::Rpc, storage::Parity};
use anyhow::Result;
use hook::DevHook;
use rpc::server::SubscriptionManager;
use runtime::{Config, Pool, Runtime, Storage};
use std::{net::SocketAddr, sync::Arc};
use sync::{config::CACHE_DIR, Sync};
use tokio::sync::{mpsc, Mutex};
use zcore::ex::Bridge;

mod author;
mod genesis;
mod hook;
mod relay;

/// The development node implementation
pub struct Dev {
    /// The storage of the development node
    pub parity: Arc<Parity>,

    /// The pool of the development node
    pub pool: Arc<Mutex<Pool>>,

    /// The RPC service
    pub rpc: Rpc<Parity>,

    /// The runtime
    pub runtime: Runtime<Development>,
}

impl Dev {
    /// Create a new development node
    pub async fn new() -> Result<Self> {
        let manager = SubscriptionManager::default();
        let hook = DevHook::new(manager.clone());
        let parity = Arc::new(Parity::try_from(CACHE_DIR.join("chain"))?);
        let runtime = Runtime::new(hook, parity.clone(), 1).await?;
        let pool = runtime.pool.clone();
        let rpc = Rpc::new(parity.clone(), manager);
        if parity.is_empty()? {
            parity.commit(genesis::commit()?)?;
        }
        Ok(Self {
            runtime,
            pool,
            rpc,
            parity,
        })
    }

    /// Start the development node
    pub async fn start(self, address: SocketAddr) -> Result<()> {
        tracing::info!("Starting the development node");
        let Dev {
            parity,
            pool,
            rpc,
            runtime,
        } = self;
        let mut sync = Sync::load().await?;
        let (tx, rx) = mpsc::channel::<Bridge>(512);
        tokio::select! {
            r = relay::start(parity, pool, rx) => r,
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

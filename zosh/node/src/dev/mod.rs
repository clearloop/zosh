//! The development node implementation

use crate::storage::Parity;
use anyhow::Result;
use hook::DevHook;
use rpc::server::SubscriptionManager;
use runtime::{Config, Runtime};
use std::sync::Arc;
use sync::config::CACHE_DIR;

mod hook;

/// The development node implementation
pub struct Dev {
    /// The runtime
    pub runtime: Runtime<Development>,
}

impl Dev {
    /// Create a new development node
    pub async fn new() -> Result<Self> {
        let manager = SubscriptionManager::default();
        let hook = DevHook::new(manager.clone());
        let parity = Arc::new(Parity::try_from(CACHE_DIR.join("chain"))?);
        let runtime = Runtime::new(hook, parity.clone()).await?;

        Ok(Self { runtime })
    }

    /// Start the development node
    pub async fn start(self) -> Result<()> {
        Ok(())
    }
}

/// The development node configuration
pub struct Development;

impl Config for Development {
    type Hook = DevHook;
    type Storage = Arc<Parity>;
}

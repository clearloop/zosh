//! The development node implementation

use crate::node::Development;
use anyhow::Result;
use rpc::server::SubscriptionManager;
use runtime::{Hook, Runtime};
use zcore::Block;

/// The development node implementation
pub struct Dev {
    /// The runtime
    pub runtime: Runtime<Development>,
}

/// The development hook
pub struct DevHook {
    manager: SubscriptionManager,
}

impl DevHook {
    /// Create a new development hook
    pub fn new(manager: SubscriptionManager) -> Self {
        Self { manager }
    }
}

impl Hook for DevHook {
    async fn on_block_finalized(&self, block: &Block) -> Result<()> {
        self.manager.dispatch_block(block).await?;

        // TODO: subscribe the transactions together here.
        Ok(())
    }
}

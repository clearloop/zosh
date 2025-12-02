//! Hooks for the runtime

use anyhow::Result;
use core::future::Future;
use zcore::Block;

/// The hook for the runtime
pub trait Hook: Clone {
    /// The hook for the runtime
    fn on_block_finalized(&self, block: &Block) -> impl Future<Output = Result<()>>;
}

impl Hook for () {
    async fn on_block_finalized(&self, _block: &Block) -> Result<()> {
        Ok(())
    }
}

//! Hooks for the runtime

use anyhow::Result;

/// The hook for the runtime
pub trait Hook {
    /// The hook for the runtime
    fn on_block_finalized(&self) -> Result<()>;
}

impl Hook for () {
    fn on_block_finalized(&self) -> Result<()> {
        Ok(())
    }
}

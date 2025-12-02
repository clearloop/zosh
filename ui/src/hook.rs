//! Hooks for the UI

use crate::db::Database;
use anyhow::Result;
use runtime::Hook;
use zcore::Block;

/// The hook for the UI
#[derive(Clone)]
pub struct UIHook {
    pub db: Database,
}

impl UIHook {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Hook for UIHook {
    async fn on_block_finalized(&self, block: &Block) -> Result<()> {
        self.db.insert_block(block)
    }
}

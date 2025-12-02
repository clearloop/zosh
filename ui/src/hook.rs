//! Hooks for the UI

use crate::db::{Database, Stats};
use anyhow::Result;
use runtime::Hook;
use tokio::sync::broadcast;
use zcore::Block;

/// The hook for the UI
#[derive(Clone)]
pub struct UIHook {
    pub db: Database,
    pub stats_tx: broadcast::Sender<Stats>,
}

impl UIHook {
    pub fn new(db: Database, stats_tx: broadcast::Sender<Stats>) -> Self {
        Self { db, stats_tx }
    }
}

impl Hook for UIHook {
    async fn on_block_finalized(&self, block: &Block) -> Result<()> {
        self.db.insert_block(block)?;

        // Broadcast updated stats to WebSocket subscribers
        if let Ok(stats) = self.db.get_stats() {
            // Ignore send errors (no subscribers)
            let _ = self.stats_tx.send(stats);
        }

        Ok(())
    }
}

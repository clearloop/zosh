//! Block subscriber module

use crate::db::Database;
use anyhow::Result;
use jsonrpsee::ws_client::WsClientBuilder;
use rpc::ApiClient;

/// Block subscriber that listens to new blocks from the RPC
pub struct Subscriber {
    rpc_url: String,
    db: Database,
}

impl Subscriber {
    /// Create a new subscriber
    pub fn new(rpc_url: String, db: Database) -> Self {
        Self { rpc_url, db }
    }

    /// Start subscribing to blocks
    pub async fn run(&self) -> Result<()> {
        tracing::info!("Connecting to Zosh RPC at {}", self.rpc_url);

        // Create WebSocket client
        let client = WsClientBuilder::default().build(&self.rpc_url).await?;
        tracing::info!("Connected to Zosh RPC, subscribing to blocks...");

        // Subscribe to blocks
        let mut subscription = client.subscribe_block().await?;
        while let Some(Ok(block)) = subscription.next().await {
            tracing::info!("Received block at slot {}", block.header.slot);

            // Count transactions
            let bridge_count: usize = block
                .extrinsic
                .bridge
                .values()
                .map(|bundle| bundle.bridge.len())
                .sum();
            let receipt_count = block.extrinsic.receipts.len();

            if bridge_count > 0 || receipt_count > 0 {
                tracing::info!(
                    "Block {} contains {} bridge transactions and {} receipts",
                    block.header.slot,
                    bridge_count,
                    receipt_count
                );
            }

            // Insert block into database
            if let Err(e) = self.db.insert_block(&block) {
                tracing::error!("Failed to insert block {}: {:?}", block.header.slot, e);
            } else {
                tracing::debug!("Successfully inserted block {}", block.header.slot);
            }
        }

        tracing::warn!("Block subscription ended");
        Ok(())
    }
}

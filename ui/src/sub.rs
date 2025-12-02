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
            let block = block.into_block()?;
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

pub mod ui {
    //! Query ID for matching the UI

    use crate::db::Database;
    use anyhow::Result;
    use std::time::Duration;
    use sync::{
        zcash::{TxId, ZcashClient},
        Sync,
    };
    use tokio::sync::mpsc;

    /// Subscribe from the light client for matching
    ///
    /// queryId with txID
    pub async fn subscribe(db: Database) -> Result<()> {
        let zcash = Sync::load().await?.zcash;
        let (tx, mut rx) = mpsc::channel::<(Vec<u8>, TxId)>(512);
        tokio::spawn(async move { zcash_sub(zcash, tx.clone()).await });
        while let Some((query_id, tx_id)) = rx.recv().await {
            if let Err(e) = db.insert_query_id(query_id, tx_id.as_ref()) {
                tracing::error!("Failed to insert query ID: {:?}", e);
            }
        }
        Ok(())
    }

    /// Subscribe from the zcash light client for matching
    pub async fn zcash_sub(mut zcash: ZcashClient, tx: mpsc::Sender<(Vec<u8>, TxId)>) {
        loop {
            if let Err(e) = zcash.dev_builder_subscribe(tx.clone()).await {
                tracing::error!("Zcash subscription error: {:?}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

//! Subscription handlers for the Spacejam JSON RPC API.

use anyhow::Result;
use jsonrpsee::{SubscriptionMessage, SubscriptionSink};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use zcore::Block;

/// The subscription type
pub type SubscriptionFilter<T> = Arc<Mutex<Vec<(T, SubscriptionSink)>>>;

/// The raw subscription type
pub type Subscription = Arc<Mutex<Vec<SubscriptionSink>>>;

/// Subscription manager
#[derive(Default, Clone)]
pub struct SubscriptionManager {
    /// The best block subscription sinks
    pub block_sub: Subscription,

    /// The transaction subscription sinks
    pub transaction_sub: SubscriptionFilter<Vec<u8>>,
}

impl SubscriptionManager {
    /// Dispatch the best block
    pub async fn dispatch_block(&self, block: &Block) -> Result<()> {
        let bytes = postcard::to_allocvec(block)?;
        let raw_value = serde_json::value::to_raw_value(&json!( {
            "block": bytes,
        }))?;
        let mut remove = Vec::new();
        for sink in self.block_sub.lock().await.iter() {
            if let Err(e) = sink
                .send(SubscriptionMessage::from(raw_value.clone()))
                .await
            {
                tracing::warn!("Failed to send block to sink: {e:?}");
                remove.push(sink.subscription_id());
            }
        }

        for id in remove {
            self.block_sub
                .lock()
                .await
                .retain(|sink| sink.subscription_id() != id);
        }
        Ok(())
    }

    // /// Dispatch the transaction status
    // pub async fn dispatch_transaction(&self, txid: &Vec<u8>) -> Result<()> {
    //     let raw_value = serde_json::value::to_raw_value(&txid)?;
    //     for sink in self.transaction_sub.lock().await.iter() {
    //         sink.send(SubscriptionMessage::from(raw_value.clone()))
    //             .await?;
    //     }
    //     Ok(())
    // }
}

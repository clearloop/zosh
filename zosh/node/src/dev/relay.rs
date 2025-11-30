//! The development node services

use crate::storage::Parity;
use anyhow::Result;
use runtime::{Pool, Storage};
use std::{mem, sync::Arc, time::Instant};
use sync::Sync;
use tokio::sync::{mpsc, Mutex};
use zcore::ex::Bridge;

// The interval to bundle the transactions in seconds
const BUNDLE_INTERVAL: u64 = 3;

/// Start the relay service
pub async fn start(
    parity: Arc<Parity>,
    pool: Arc<Mutex<Pool>>,
    mut rx: mpsc::Receiver<Bridge>,
) -> Result<()> {
    let mut sync = Sync::load().await?;
    let mut now = Instant::now();
    let mut bridges = Vec::new();
    while let Some(bridge) = rx.recv().await {
        // skip if the transaction is already processed
        if parity.exists(&bridge.txid)? {
            continue;
        }

        // validate the bridge request
        //
        // TODO: in production we should do this in parallel.
        if let Err(e) = sync.validate_bridge(&bridge).await {
            tracing::error!("{e:?}");
            continue;
        }

        // Do the validation of the bridge request, insert to the queue
        // if it is valid.
        bridges.push(bridge);
        if now.elapsed().as_secs() < BUNDLE_INTERVAL {
            continue;
        }

        let (bundles, receipts) = sync.bundle(mem::take(&mut bridges)).await?;
        let mut pool = pool.lock().await;
        pool.bridge.queue(bundles)?;
        pool.receipt.extend_from_slice(receipts.as_slice());
        now = Instant::now();
    }
    Ok(())
}

//! The development node services

use crate::storage::Parity;
use anyhow::Result;
use runtime::{Pool, Storage};
use std::{
    mem,
    sync::Arc,
    time::{Duration, Instant},
};
use sync::Sync;
use tokio::sync::{mpsc, Mutex};
use zcore::ex::Bridge;

// The interval to bundle the transactions in seconds
const BUNDLE_INTERVAL: u64 = 3;

/// One second
const ONE_SECOND: Duration = Duration::from_secs(1);

/// Start the relay service
pub async fn start(
    parity: Arc<Parity>,
    pool: Arc<Mutex<Pool>>,
    rx: mpsc::Receiver<Bridge>,
) -> Result<()> {
    tracing::info!("Starting the relay service");
    let sync = Arc::new(Mutex::new(Sync::load().await?));
    let bridges = Arc::new(Mutex::new(Vec::new()));

    tokio::select! {
        r = collector(parity, rx, sync.clone(), bridges.clone()) => r,
        r = bundler(sync, bridges, pool) => r,
    }
}

async fn collector(
    parity: Arc<Parity>,
    mut rx: mpsc::Receiver<Bridge>,
    sync: Arc<Mutex<Sync>>,
    bridges: Arc<Mutex<Vec<Bridge>>>,
) -> Result<()> {
    while let Some(bridge) = rx.recv().await {
        tracing::info!("Received bridge request: {:?}", bridge);
        // skip if the transaction is already processed
        if parity.exists(&bridge.txid)? {
            tracing::warn!("Bridge request already processed: {:?}", bridge.txid);
            continue;
        }

        // validate the bridge request
        //
        // TODO: in production we should do this in parallel.
        if let Err(e) = sync.lock().await.validate_bridge(&bridge).await {
            tracing::error!("{e:?}");
            continue;
        }

        // Do the validation of the bridge request, insert to the queue
        // if it is valid.
        bridges.lock().await.push(bridge);
    }
    Ok(())
}

async fn bundler(
    sync: Arc<Mutex<Sync>>,
    bridges: Arc<Mutex<Vec<Bridge>>>,
    pool: Arc<Mutex<Pool>>,
) -> Result<()> {
    let mut now = Instant::now();
    loop {
        if now.elapsed().as_secs() < BUNDLE_INTERVAL {
            tokio::time::sleep(ONE_SECOND).await;
            continue;
        }

        let mut bridges = bridges.lock().await;
        if bridges.is_empty() {
            continue;
        }

        let mut sync = sync.lock().await;
        tracing::info!("Bundling {} bridge requests", bridges.len());
        let (bundles, receipts) = sync.bundle(mem::take(&mut bridges)).await?;
        let mut pool = pool.lock().await;
        pool.bridge.queue(bundles)?;
        pool.receipt.extend_from_slice(&receipts);
        now = Instant::now();
    }
}

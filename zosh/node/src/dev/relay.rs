//! The development node services

use crate::storage::Parity;
use anyhow::Result;
use runtime::{Pool, Storage};
use std::{
    mem,
    sync::Arc,
    time::{Duration, Instant},
};
use sync::{zcash::Network, ChainFormatEncoder, Sync};
use tokio::sync::{mpsc, Mutex};
use zcore::{ex::Bridge, registry::Chain};

// The interval to bundle the transactions in seconds
const BUNDLE_INTERVAL: u64 = 3;

/// One second
const ONE_SECOND: Duration = Duration::from_secs(1);

/// Start the relay service
pub async fn spawn(
    parity: Arc<Parity>,
    pool: Arc<Mutex<Pool>>,
    rx: mpsc::Receiver<Bridge>,
) -> Result<()> {
    let sync = Arc::new(Mutex::new(Sync::load().await?));
    let bridges = Arc::new(Mutex::new(Vec::new()));

    // spawn the bundler service
    let sync2 = sync.clone();
    let bridges2 = bridges.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = bundler(sync2.clone(), bridges2.clone(), pool.clone()).await {
                tracing::error!("Bundler error: {e:?}, retrying in 5 seconds");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });

    tokio::spawn(async move { collector(parity, rx, sync.clone(), bridges.clone()).await });
    Ok(())
}

async fn collector(
    parity: Arc<Parity>,
    mut rx: mpsc::Receiver<Bridge>,
    sync: Arc<Mutex<Sync>>,
    bridges: Arc<Mutex<Vec<Bridge>>>,
) -> Result<()> {
    while let Some(bridge) = rx.recv().await {
        if let Err(e) = collector_internal(&parity, &sync, &bridges, bridge).await {
            tracing::error!("{e:?}");
        }

        continue;
    }
    Ok(())
}

async fn collector_internal(
    parity: &Arc<Parity>,
    sync: &Arc<Mutex<Sync>>,
    bridges: &Arc<Mutex<Vec<Bridge>>>,
    bridge: Bridge,
) -> Result<()> {
    // skip if the transaction is already processed
    if parity.exists(&bridge.txid)? {
        return Ok(());
    }

    // validate the bridge request
    //
    // TODO: in production we should do this in parallel.
    sync.lock().await.validate_bridge(&bridge).await?;

    // print the bridge request details
    tracing::info!(
        "Received bridge request: from {:?} to {:?}({}) with amount {}, txid={}",
        bridge.source,
        bridge.target,
        match bridge.target {
            Chain::Solana => bridge.recipient.solana_address()?.to_string(),
            Chain::Zcash => format!(
                "{:?}",
                bridge.recipient.zcash_address(&Network::TestNetwork)?
            ),
        },
        bridge.amount,
        match bridge.source {
            Chain::Solana => bridge.txid.solana_signature()?.to_string(),
            Chain::Zcash => bridge.txid.zcash_signature()?.to_string(),
        }
    );

    // Do the validation of the bridge request, insert to the queue
    // if it is valid.
    bridges.lock().await.push(bridge);
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
        let (bundles, receipts) = sync.bundle(mem::take(&mut bridges)).await?;
        let mut pool = pool.lock().await;
        if !bundles.is_empty() {
            pool.bridge.dev_pack(bundles)?;
        }
        if !receipts.is_empty() {
            pool.receipt.extend_from_slice(&receipts);
        }
        now = Instant::now();
    }
}

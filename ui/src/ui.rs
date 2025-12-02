//! Query ID for matching the UI

use std::time::Duration;

use anyhow::Result;
use sync::{
    zcash::{TxId, ZcashClient},
    Sync,
};
use tokio::sync::mpsc;

use crate::db::Database;

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

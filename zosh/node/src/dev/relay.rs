//! The development node services

use anyhow::Result;
use runtime::Pool;
use std::sync::Arc;
use sync::Event;
use tokio::sync::{mpsc, Mutex};

/// Start the relay service
pub async fn start(pool: Arc<Mutex<Pool>>, mut rx: mpsc::Receiver<Event>) -> Result<()> {
    while let Some(event) = rx.recv().await {
        match event {
            Event::Bridge(bridge) => pool.lock().await.bridge.add(bridge),
            Event::Receipt(receipt) => pool.lock().await.receipt.push(receipt),
        };
    }

    Ok(())
}

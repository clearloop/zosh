//! Command for the POC service

use anyhow::Result;
use std::path::Path;
use sync::{Config, Event, Sync};
use tokio::sync::mpsc;
use zcore::tx::Chain;

/// Run the POC service
pub async fn run(cache: &Path, config: &Config) -> Result<()> {
    let sync = Sync::new(cache, config).await?;
    let (tx, rx) = mpsc::channel::<Event>(512);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => Ok(()),
        r = sync.start(tx.clone()) => r,
        r = handle(rx) => r,
    }
}

async fn handle(mut rx: mpsc::Receiver<Event>) -> Result<()> {
    loop {
        while let Some(event) = rx.recv().await {
            match event {
                Event::Bridge(bridge) => {
                    tracing::info!(
                        "Received bridge request: target={:?}, recipient={}, amount={}",
                        bridge.target,
                        match bridge.target {
                            Chain::Solana => bs58::encode(bridge.recipient).into_string(),
                            Chain::Zcash => String::from_utf8(bridge.recipient).unwrap(),
                        },
                        bridge.amount as f32 / 100_000_000.0
                    );
                }
                _ => {}
            }
        }
    }
}

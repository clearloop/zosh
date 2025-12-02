//! The development node authoring service

use crate::dev::Development;
use anyhow::Result;
use runtime::{Hook, Runtime};
use solana_signer::Signer;
use std::time::{Duration, Instant};
use sync::solana::dev;

// The interval to author the block in seconds
const AUTHOR_INTERVAL: u64 = 3;

/// One second
const ONE_SECOND: Duration = Duration::from_secs(1);

/// Start the authoring service
///
/// - use the solana keyper as signer
/// - The current node is always the leader.
pub async fn start(mut runtime: Runtime<Development>) -> Result<()> {
    let mut now = Instant::now();
    let authority = dev::load_authority()?;
    let ident = authority.pubkey().to_bytes();

    loop {
        if now.elapsed().as_secs() < AUTHOR_INTERVAL {
            tokio::time::sleep(ONE_SECOND).await;
            continue;
        }

        let mut block = runtime.author().await?;
        let slot = block.header.slot;
        let hash = block.header.hash();
        let signature = authority.sign_message(&hash);
        block
            .header
            .votes
            .insert(ident, signature.as_array().to_vec());

        tracing::debug!(
            "Imported block: slot={slot} hash={} bundles={} receipts={}",
            bs58::encode(&hash).into_string(),
            block.extrinsic.bridge.len(),
            block.extrinsic.receipts.len()
        );
        runtime.import(&block)?;
        runtime.hook.on_block_finalized(&block).await?;
        now = Instant::now();
    }
}

/// Spawn the authoring service
pub fn spawn(runtime: Runtime<Development>) -> Result<()> {
    tokio::spawn(async move {
        loop {
            if let Err(e) = start(runtime.clone()).await {
                tracing::error!("authoring service error:{e:?}, restarting in 5 seconds");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });
    Ok(())
}

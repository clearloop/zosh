//! The development node authoring service

use crate::dev::Development;
use anyhow::Result;
use runtime::Runtime;
use solana_signer::Signer;
use std::time::Instant;
use sync::solana::dev;

// The interval to author the block in seconds
const AUTHOR_INTERVAL: u64 = 3;

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

        runtime.import(block)?;
        tracing::debug!(
            "Imported block: slot={slot} hash={}",
            bs58::encode(&hash).into_string()
        );
        now = Instant::now();
    }
}

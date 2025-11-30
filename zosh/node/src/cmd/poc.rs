//! Command for the POC service

use anyhow::Result;
use sync::{
    solana::{self, Pubkey},
    zcash::{self, AddressCodec, UnifiedAddress},
    Config, Event, Sync,
};
use tokio::sync::mpsc;
use zcore::registry::Chain;

/// Run the POC service
pub async fn run(config: &Config) -> Result<()> {
    let mut sync = Sync::new(config).await?;
    let sync2 = Sync::new(config).await?;
    let zcash: zcash::GroupSigners =
        postcard::from_bytes(&bs58::decode(config.key.zcash.as_str()).into_vec()?)?;
    let solana: solana::GroupSigners =
        postcard::from_bytes(&bs58::decode(config.key.solana.as_str()).into_vec()?)?;
    let (tx, rx) = mpsc::channel::<Event>(512);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => Ok(()),
        r = sync.start(tx.clone()) => r,
        r = handle(solana, zcash, sync2, rx) => r,
    }
}

async fn handle(
    solana: solana::GroupSigners,
    zcash: zcash::GroupSigners,
    mut sync: Sync,
    mut rx: mpsc::Receiver<Event>,
) -> Result<()> {
    while let Some(event) = rx.recv().await {
        let Event::Bridge(bridge) = event else {
            continue;
        };

        tracing::info!(
            "Received bridge request: target={:?}, recipient={}, amount={}",
            bridge.target,
            match bridge.target {
                Chain::Solana => bs58::encode(&bridge.recipient).into_string(),
                Chain::Zcash => String::from_utf8(bridge.recipient.clone()).unwrap(),
            },
            bridge.amount as f32 / 100_000_000.0
        );

        match bridge.target {
            Chain::Solana => {
                let mut pubkey = [0; 32];
                pubkey.copy_from_slice(bridge.recipient.as_slice());
                sync.solana
                    .dev_mint(Pubkey::new_from_array(pubkey), bridge.amount, &solana)
                    .await?;
            }
            Chain::Zcash => {
                let address = UnifiedAddress::decode(
                    &sync.zcash.network,
                    &String::from_utf8(bridge.recipient)?,
                )
                .map_err(|e| anyhow::anyhow!(e))?;
                sync.zcash.dev_send(&zcash, address, bridge.amount).await?;
            }
        }
    }

    Ok(())
}

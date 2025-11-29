//! The subscription of the solana client

use crate::{solana::SolanaClient, Event};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use futures_util::StreamExt;
use solana_rpc_client_types::{
    config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
    response::RpcLogsResponse,
};
use solana_sdk::commitment_config::CommitmentConfig;
use tokio::sync::mpsc;
use zcore::{
    ex::Bridge,
    registry::{Chain, Coin},
};
use zosh::{
    client::{AnchorDeserialize, Discriminator},
    BurnEvent, MintEvent,
};

impl SolanaClient {
    /// Subscribe to the solana client
    pub async fn subscribe(&self, tx: mpsc::Sender<Event>) -> Result<()> {
        let filter = RpcTransactionLogsFilter::Mentions(vec![zosh::ID.to_string()]);
        let config = RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::confirmed()),
        };

        let (mut noti, unsubscribe) = self.sub.logs_subscribe(filter, config).await?;
        while let Some(response) = noti.next().await {
            let RpcLogsResponse {
                signature,
                err,
                logs,
            } = response.value;
            if err.is_some() {
                continue;
            }

            // Parse events from logs
            //
            // TODO: shall we embed slot in the event as well?
            for entry in &logs {
                if handle_event(tx.clone(), entry, signature.clone())
                    .await
                    .is_err()
                {
                    continue;
                }
            }
        }

        unsubscribe().await;
        tracing::info!("Solana log subscription for program {} closed", zosh::ID);
        Ok(())
    }
}

/// Parse an Anchor event from a Solana program log entry
async fn handle_event(tx: mpsc::Sender<Event>, log: &str, signature: String) -> Result<()> {
    let log = log
        .trim_start_matches("Program log: ")
        .trim_start_matches("Program data: ");

    let bytes = STANDARD.decode(log.trim())?;
    if bytes.len() < 8 {
        anyhow::bail!("Invalid log length");
    }

    let data = &mut &bytes[8..];
    match &bytes[..8] {
        BurnEvent::DISCRIMINATOR => {
            let burn = BurnEvent::deserialize(data)?;
            tracing::debug!(
                "Received bridge request target={}, amount={}",
                burn.zec_recipient,
                burn.amount
            );
            tx.send(Event::Bridge(Bridge {
                coin: Coin::Zec,
                recipient: burn.zec_recipient.into(),
                amount: burn.amount,
                txid: bs58::decode(signature).into_vec()?,
                source: Chain::Solana,
                target: Chain::Zcash,
            }))
            .await?;
        }
        MintEvent::DISCRIMINATOR => {
            // TODO: probably we don't need prompt the receipt event
            // here, it should be fetched on confirming the transaction.
            let mint = MintEvent::deserialize(data)?;
            for (recipient, amount) in mint.mints {
                tracing::debug!(
                    "Received mint event: recipient={}, amount={}",
                    recipient,
                    amount
                );
            }
        }
        _ => {}
    }

    Ok(())
}

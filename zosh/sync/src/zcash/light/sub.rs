//! The subscription of the zcash light client

use crate::zcash::ZcashClient;
use anyhow::Result;
use orchard::keys::Scope;
use std::time::Duration;
use tokio::{sync::mpsc, time};
use zcash_client_backend::{
    fees::orchard::InputView,
    proto::service::{BlockId, TxFilter},
};
use zcash_primitives::transaction::Transaction;
use zcash_protocol::{
    consensus::{BlockHeight, BranchId},
    memo::{Memo, MemoBytes},
    TxId,
};
use zcore::{
    ex::Bridge,
    registry::{Chain, Coin},
};

/// The block time of zcash in seconds
pub const ZCASH_BLOCK_TIME: u64 = 75;

impl ZcashClient {
    /// Subscribe to the zcash light client
    ///
    /// FIXME: write new query of the walletdb to fetch the
    /// latest transactions efficiently.
    pub async fn subscribe(&mut self, tx: mpsc::Sender<Bridge>) {
        loop {
            if let Err(e) = self.subscribe_inner(tx.clone()).await {
                tracing::error!(
                    "Zcash light client subscription error:{e:?}, retrying in 5 seconds"
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }

    /// Subscribe to the zcash light client
    ///
    /// Note: Deduplication is handled by the relay layer using the database,
    /// so we simply send all spendable notes. This avoids memory issues from
    /// tracking processed txids and ensures no notes are missed.
    pub async fn subscribe_inner(&mut self, tx: mpsc::Sender<Bridge>) -> Result<()> {
        loop {
            self.sync().await?;
            let Ok((target, _anchor)) = self.heights() else {
                tracing::error!(
                    "Failed to get max height and hash, retrying in {} seconds",
                    ZCASH_BLOCK_TIME
                );
                time::sleep(Duration::from_secs(ZCASH_BLOCK_TIME)).await;
                continue;
            };
            tracing::trace!("zcash light synced to height {}", u32::from(target));

            // Get the spendable notes and send them all.
            // The relay layer deduplicates using parity.exists(&bridge.txid).
            let notes = self.spendable_notes(0, target, &self.blacklist)?;
            for note in notes.into_iter() {
                let txid = note.txid();
                let Some(mined_height) = note.mined_height() else {
                    tracing::warn!("Note mined height is not found for note of {}", &txid);
                    self.blacklist.push(*note.internal_note_id());
                    continue;
                };

                let Ok(Memo::Text(text)) = self
                    .fetch_memo(mined_height, *txid, note.output_index() as u32)
                    .await
                    .inspect_err(|e| {
                        tracing::warn!("Failed to fetch memo for note of {}: {:?}", &txid, e);
                    })
                else {
                    // TODO: raise a refund event to the node.
                    self.blacklist.push(*note.internal_note_id());
                    continue;
                };

                // split the memo into parts
                let parts = text.trim().split(':').collect::<Vec<&str>>();
                if parts.is_empty() {
                    self.blacklist.push(*note.internal_note_id());
                    continue;
                }

                // NOTE: support solana only
                //
                // TODO: check the address length is valid, if not, introduce
                // a refund transaction to the node, since we'll recheck it
                // on sending the transaction, not doing it here for now.
                //
                // TODO: for non-bridge memo, we should blacklist them.
                let Ok(recipient) = bs58::decode(parts[0]).into_vec() else {
                    self.blacklist.push(*note.internal_note_id());
                    continue;
                };

                // NOTE: Invalid recipient address, skip it.
                if recipient.len() != 32 {
                    self.blacklist.push(*note.internal_note_id());
                    continue;
                }

                // NOTE: we support solana address only here, for the bytes
                // after 32, they will be used for the builders to enhance
                // the user experience.
                tx.send(Bridge {
                    coin: Coin::Zec,
                    recipient,
                    amount: note.value().into_u64(),
                    txid: txid.as_ref().to_vec(),
                    source: Chain::Zcash,
                    target: Chain::Solana,
                })
                .await?;
            }

            // The block time of zcash is 75 secs, using 10 secs is fine here.
            time::sleep(Duration::from_secs(10)).await;
        }
    }

    /// Subscribe to the zcash light client for builder data
    ///
    /// Note: This is used by the UI layer which has its own deduplication
    /// via insert_query_id's INSERT OR REPLACE.
    pub async fn dev_builder_subscribe(&mut self, tx: mpsc::Sender<(Vec<u8>, TxId)>) -> Result<()> {
        loop {
            // The block time of zcash is 75 secs, using 10 secs is fine here.
            time::sleep(Duration::from_secs(10)).await;
            let Ok((target, _anchor)) = self.heights() else {
                tracing::error!(
                    "Failed to get max height and hash, retrying in {} seconds",
                    ZCASH_BLOCK_TIME
                );
                continue;
            };

            // Get the spendable notes
            let notes = self.spendable_notes(0, target, &[])?;
            for note in notes.into_iter() {
                let txid = note.txid();
                let Some(mined_height) = note.mined_height() else {
                    tracing::warn!("Note mined height is not found for note of {}", &txid);
                    self.blacklist.push(*note.internal_note_id());
                    continue;
                };

                let Ok(Memo::Text(text)) = self
                    .fetch_memo(mined_height, *txid, note.output_index() as u32)
                    .await
                    .inspect_err(|e| {
                        tracing::warn!("Failed to fetch memo for note of {}: {:?}", &txid, e);
                    })
                else {
                    self.blacklist.push(*note.internal_note_id());
                    continue;
                };

                let parts = text.trim().split(':').collect::<Vec<&str>>();
                if parts.len() != 2 {
                    self.blacklist.push(*note.internal_note_id());
                    continue;
                }

                let Ok(data) = bs58::decode(parts[1]).into_vec() else {
                    self.blacklist.push(*note.internal_note_id());
                    continue;
                };

                tx.send((data, *txid)).await?;
            }
        }
    }

    /// TODO: introduce memory cache for this or flush it to
    /// the walletdb.
    async fn fetch_memo(
        &mut self,
        height: BlockHeight,
        txid: TxId,
        output_index: u32,
    ) -> Result<Memo> {
        let block = self
            .client
            .get_block(BlockId {
                height: height.into(),
                hash: Default::default(),
            })
            .await?
            .into_inner();
        let mut index = 0;
        for (idx, tx) in block.vtx.iter().enumerate() {
            if tx.txid() == txid {
                index = idx;
                break;
            }
        }

        let rawtx = self
            .client
            .get_transaction(TxFilter {
                block: Some(BlockId {
                    height: height.into(),
                    hash: Default::default(),
                }),
                index: index as u64,
                hash: txid.as_ref().to_vec(),
            })
            .await?
            .into_inner();
        let tx = Transaction::read(
            rawtx.data.as_slice(),
            BranchId::for_height(&self.network, height),
        )?;

        let memo = tx
            .orchard_bundle()
            .ok_or(anyhow::anyhow!("Failed to get orchard bundle"))?
            .decrypt_output_with_key(
                output_index as usize,
                &self
                    .ufvk
                    .orchard()
                    .ok_or(anyhow::anyhow!("Failed to get orchard full viewing key"))?
                    .to_ivk(Scope::External),
            )
            .ok_or(anyhow::anyhow!("Failed to decrypt output"))?
            .2;

        let memo = MemoBytes::from_bytes(&memo)?;
        Ok(Memo::try_from(memo)?)
    }
}

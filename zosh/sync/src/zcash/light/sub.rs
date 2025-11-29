//! The subscription of the zcash light client

use crate::{zcash::Light, Event};
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

impl Light {
    /// Subscribe to the zcash light client
    ///
    /// FIXME: write new query of the walletdb to fetch the
    /// latest transactions efficiently.
    pub async fn subscribe(&mut self, tx: mpsc::Sender<Event>) -> Result<()> {
        // TODO: we should get the latest height from the global on-chain state.
        let mut last_height = BlockHeight::from(0);
        loop {
            let Ok((target, _anchor)) = self.heights() else {
                tracing::error!(
                    "Failed to get max height and hash, retrying in {} seconds",
                    ZCASH_BLOCK_TIME
                );
                time::sleep(Duration::from_secs(ZCASH_BLOCK_TIME)).await;
                continue;
            };

            // Get the spendable notes
            let notes = self.spendable_notes(0, target)?;
            for note in notes.into_iter() {
                let txid = note.txid();
                let Some(mined_height) = note.mined_height() else {
                    tracing::warn!("Note mined height is not found for note of {}", &txid);
                    continue;
                };

                if mined_height <= last_height {
                    continue;
                }

                let Ok(Memo::Text(text)) = self
                    .fetch_memo(mined_height, *txid, note.output_index() as u32)
                    .await
                    .inspect_err(|e| {
                        tracing::warn!("Failed to fetch memo for note of {}: {:?}", &txid, e);
                    })
                else {
                    // TODO: raise a refund event to the node.
                    // tracing::warn!("Memo is not found for note of {}", &txid);
                    continue;
                };

                // NOTE: support solana only
                //
                // TODO: check the address length is valid, if not, introduce
                // a refund transaction to the node, since we'll recheck it
                // on sending the transaction, not doing it here for now.
                tracing::debug!(
                    "Received bridge memo={}, amount={}",
                    &text.to_string().trim(),
                    note.value().into_u64() as f32 / 100_000_000.0
                );
                let recipient = bs58::decode(text.trim()).into_vec()?;
                tx.send(Event::Bridge(Bridge {
                    coin: Coin::Zec,
                    recipient,
                    amount: note.value().into_u64(),
                    txid: txid.as_ref().to_vec(),
                    source: Chain::Zcash,
                    target: Chain::Solana,
                }))
                .await?;
            }

            // The block time of zcash is 75 secs, using 30 secs is totally fine here.
            last_height = target.into();
            time::sleep(Duration::from_secs(30)).await;
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

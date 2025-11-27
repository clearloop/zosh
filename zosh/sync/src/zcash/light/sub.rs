//! The subscription of the zcash light client

use crate::{zcash::Light, Event};
use anyhow::Result;
use std::time::Duration;
use tokio::{sync::mpsc, time};
use zcash_client_backend::{
    data_api::{wallet::ConfirmationsPolicy, Account, InputSource, TargetValue, WalletRead},
    fees::orchard::InputView,
    wallet::NoteId,
};
use zcash_protocol::{consensus::BlockHeight, memo::Memo, value::Zatoshis, ShieldedProtocol};
use zcore::tx::{Bridge, Chain};

/// The block time of zcash in seconds
pub const ZCASH_BLOCK_TIME: u64 = 75;

impl Light {
    /// Subscribe to the zcash light client
    ///
    /// FIXME: write new query of the walletdb to fetch the
    /// latest transactions efficiently.
    pub async fn subscribe(&self, tx: mpsc::Sender<Event>) -> Result<()> {
        // TODO: we should get the latest height from the global on-chain state.
        let mut last_height = BlockHeight::from(0);
        let confirmations = ConfirmationsPolicy::default();
        loop {
            let Some((target, _anchor)) = self
                .wallet
                .get_target_and_anchor_heights(confirmations.trusted())
                .map_err(|e| anyhow::anyhow!("Failed to get max height and hash: {:?}", e))?
            else {
                tracing::error!(
                    "Failed to get max height and hash, retrying in {} seconds",
                    ZCASH_BLOCK_TIME
                );
                time::sleep(Duration::from_secs(ZCASH_BLOCK_TIME)).await;
                continue;
            };

            let Some(account) = self.wallet.get_account_for_ufvk(&self.ufvk)? else {
                return Err(anyhow::anyhow!("Account not found by provided ufvk"));
            };

            // Get the spendable notes
            let notes = self.wallet.select_spendable_notes(
                account.id(),
                TargetValue::AtLeast(Zatoshis::from_u64(0)?),
                &[ShieldedProtocol::Orchard],
                target,
                confirmations,
                &[],
            )?;

            for note in notes.orchard() {
                let txid = note.txid();
                let Some(mined_height) = note.mined_height() else {
                    tracing::warn!("Note mined height is not found for note of {}", &txid);
                    continue;
                };

                if mined_height <= last_height {
                    continue;
                }

                let note_id = NoteId::new(
                    txid.clone(),
                    ShieldedProtocol::Orchard,
                    note.output_index() as u16,
                );
                let Some(Memo::Text(text)) = self.wallet.get_memo(note_id)? else {
                    // TODO: raise a refund event to the node.
                    tracing::warn!("Memo is not found for note of {}", &txid);
                    continue;
                };

                // NOTE: support solana only
                //
                // TODO: check the address length is valid, if not, introduce
                // a refund transaction to the node, since we'll recheck it
                // on sending the transaction, not doing it here for now.
                let recipient = bs58::decode(text.trim()).into_vec()?;
                tx.send(Event::Bridge(Bridge {
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
}

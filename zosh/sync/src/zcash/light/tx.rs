//! Transaction related interfaces

use crate::zcash::{light::ZcashClient, signer::GroupSigners};
use anyhow::Result;
use incrementalmerkletree::MerklePath;
use orchard::{
    builder::{self, BundleType, OutputInfo, SpendInfo},
    bundle::Flags,
    keys::Scope,
    tree::MerkleHashOrchard,
    value::NoteValue,
    Anchor, Note,
};
use zcash_client_backend::{
    data_api::WalletCommitmentTrees, fees::orchard::InputView, proto::service::RawTransaction,
    wallet::ReceivedNote,
};
use zcash_client_sqlite::ReceivedNoteId;
use zcash_keys::address::UnifiedAddress;
use zcash_primitives::transaction::{TransactionData, TxVersion, Unauthorized};
use zcash_protocol::{
    consensus::{BlockHeight, BranchId},
    value::ZatBalance,
    TxId,
};

/// ZIP-317 base fee (for up to 2 logical actions)
const ZIP317_BASE_FEE: u64 = 10_000;

/// ZIP-317 marginal fee per action beyond the first 2
const ZIP317_MARGINAL_FEE: u64 = 5_000;

/// ZIP-317 grace actions (no marginal fee for first 2 actions)
const ZIP317_GRACE_ACTIONS: usize = 2;

/// The memo for a bridged transaction
const BRIDGE_MEMO: [u8; 31] = *b"Bridged from solana via zosh.io";

/// The memo for a change output
const CHANGE_MEMO: [u8; 512] = [0; 512];

/// Calculate ZIP-317 fee for a given number of actions
/// In Orchard, num_actions = max(num_spends, num_outputs)
fn calculate_zip317_fee(num_actions: usize) -> u64 {
    if num_actions <= ZIP317_GRACE_ACTIONS {
        ZIP317_BASE_FEE
    } else {
        ZIP317_BASE_FEE + ZIP317_MARGINAL_FEE * ((num_actions - ZIP317_GRACE_ACTIONS) as u64)
    }
}

impl ZcashClient {
    /// Sign and send a transaction for development purposes
    pub async fn dev_sign_and_send(
        &mut self,
        utx: TransactionData<Unauthorized>,
        signer: &GroupSigners,
    ) -> Result<TxId> {
        let tx = signer.sign_tx(utx)?.freeze()?;
        let txid = tx.txid();
        let mut data = Vec::new();
        tx.write(&mut data)?;

        // send the transaction
        tracing::info!("Transaction ID: {}", txid);
        let resp = self
            .client
            .send_transaction(RawTransaction { data, height: 0 })
            .await?
            .into_inner();

        // Check if the transaction was successfully sent
        if resp.error_code != 0 {
            return Err(anyhow::anyhow!(
                "Transaction send failed: {} - {}",
                resp.error_code,
                resp.error_message
            ));
        }

        self.sync().await?;
        Ok(txid)
    }

    /// Send a fund to a orchard address for development purposes
    pub async fn dev_send(
        &mut self,
        signer: &GroupSigners,
        recipient: UnifiedAddress,
        amount: u64,
    ) -> Result<()> {
        let utx = self.tx(recipient, amount)?;
        let tx = signer.sign_tx(utx)?.freeze()?;
        let txid = tx.txid();
        let mut data = Vec::new();
        tx.write(&mut data)?;

        // send the transaction
        let resp = self
            .client
            .send_transaction(RawTransaction { data, height: 0 })
            .await?
            .into_inner();

        // Check if the transaction was successfully sent
        if resp.error_code != 0 {
            return Err(anyhow::anyhow!(
                "Transaction send failed: {} - {}",
                resp.error_code,
                resp.error_message
            ));
        }

        self.sync().await?;
        tracing::info!("Transaction ID: {}", txid);
        Ok(())
    }

    /// Send a fund to a orchard address for development purposes
    /// The `amount` parameter is the total to spend (including fee).
    /// The recipient will receive `amount - fee`.
    pub fn tx(
        &mut self,
        recipient: UnifiedAddress,
        amount: u64,
    ) -> Result<TransactionData<Unauthorized>> {
        let Some(recipient) = recipient.orchard() else {
            return Err(anyhow::anyhow!("Invalid orchard address"));
        };

        let Some(fvk) = self.ufvk.orchard().cloned() else {
            return Err(anyhow::anyhow!("Invalid orchard full viewing key"));
        };

        // 1. Select notes to cover the total amount (which includes fee)
        let (target_height, anchor_height) = self.heights()?;
        let notes = self.spendable_notes(amount, target_height, &[])?;
        if notes.is_empty() {
            return Err(anyhow::anyhow!("No spendable notes found"));
        }

        let total_note_value: u64 = notes.iter().map(|note| note.value().into_u64()).sum();
        let num_spends = notes.len();

        // 2. Calculate the actual number of outputs and actions
        // We'll have at least 1 output (recipient), possibly 2 if there's change
        // For now, assume we'll have change to calculate fee conservatively
        let num_outputs_with_change = 2;
        let num_actions = num_spends.max(num_outputs_with_change);
        let fee = calculate_zip317_fee(num_actions);
        if amount < fee {
            return Err(anyhow::anyhow!(
                "Amount {} is less than required fee {}",
                amount,
                fee
            ));
        }

        // Calculate the delivery amount (what recipient actually receives)
        let delivery_amount = amount
            .checked_sub(fee)
            .ok_or_else(|| anyhow::anyhow!("Amount too small to cover fee"))?;
        if total_note_value < amount {
            return Err(anyhow::anyhow!(
                "Insufficient funds: have {}, need {} (delivery: {}, fee: {})",
                total_note_value,
                amount,
                delivery_amount,
                fee
            ));
        }

        // 3. Prepare outputs: recipient output + change output (if any)
        let change = total_note_value - amount;
        let mut outputs = Vec::new();
        let mut recipient_memo = [0; 512];
        recipient_memo[..31].copy_from_slice(&BRIDGE_MEMO);
        outputs.push(OutputInfo::new(
            None,
            *recipient,
            NoteValue::from_raw(delivery_amount),
            recipient_memo,
        ));

        // If there's change, send it back to our own address
        if change > 0 {
            let change_address = fvk.address_at(0u64, Scope::External);
            outputs.push(OutputInfo::new(
                None,
                change_address,
                NoteValue::from_raw(change),
                CHANGE_MEMO,
            ));
        }

        // 3. Create SpendInfo for all notes being spent
        let mut spend_infos = Vec::new();
        let (anchor, merkle_paths) = self.merkle_path(anchor_height, &notes)?;
        for (note, merkle_path) in notes.iter().zip(merkle_paths.iter()) {
            let spend_info = SpendInfo::new(fvk.clone(), *note.note(), merkle_path.clone().into())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Failed to create spend info for note {}",
                        note.internal_note_id()
                    )
                })?;
            spend_infos.push(spend_info);
        }

        // 4. make the bundle of the transaction
        let Some((bundle, _meta)) = builder::bundle::<ZatBalance>(
            rand_core::OsRng,
            anchor,
            BundleType::Transactional {
                flags: Flags::ENABLED,
                bundle_required: false,
            },
            spend_infos,
            outputs,
        )?
        else {
            return Err(anyhow::anyhow!("Failed to create bundle"));
        };

        // Determine the correct branch ID based on the target height
        let branch_id = BranchId::for_height(&self.network, BlockHeight::from(target_height));
        let expiry_height = BlockHeight::from(target_height) + 20;
        Ok(TransactionData::<Unauthorized>::from_parts(
            TxVersion::suggested_for_branch(branch_id),
            branch_id,
            0,
            expiry_height,
            None,
            None,
            None,
            Some(bundle),
        ))
    }

    /// Get the merkle path for the notes at the given height
    fn merkle_path(
        &mut self,
        height: BlockHeight,
        notes: &[ReceivedNote<ReceivedNoteId, Note>],
    ) -> Result<(Anchor, Vec<MerklePath<MerkleHashOrchard, 32>>)> {
        self.wallet
            .with_orchard_tree_mut::<_, _, anyhow::Error>(|tree| {
                let anchor = tree.root_at_checkpoint_id(&height)?.ok_or_else(|| {
                    anyhow::anyhow!("Missing Orchard anchor at height {}", u32::from(height))
                })?;

                let mut paths = Vec::new();
                for note in notes {
                    let merkle_path = tree
                        .witness_at_checkpoint_id_caching(
                            note.note_commitment_tree_position(),
                            &height,
                        )?
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Missing Orchard witness for note {} at height {}",
                                note.internal_note_id(),
                                u32::from(height)
                            )
                        })?;
                    paths.push(merkle_path);
                }

                Ok::<_, anyhow::Error>((anchor.into(), paths))
            })
    }
}

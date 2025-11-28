//! Transaction related interfaces

use crate::zcash::{signer::GroupSigners, Light};
use anyhow::Result;
use orchard::{
    builder::{self, BundleType, OutputInfo, SpendInfo},
    bundle::Flags,
    keys::Scope,
    value::NoteValue,
};
use zcash_client_backend::{
    data_api::WalletCommitmentTrees, fees::orchard::InputView, proto::service::RawTransaction,
};
use zcash_keys::address::UnifiedAddress;
use zcash_primitives::transaction::{TransactionData, TxVersion, Unauthorized};
use zcash_protocol::{
    consensus::{BlockHeight, BranchId},
    value::ZatBalance,
};

/// The standard fee for a transaction
const STANDARD_FEE: u64 = 1000;

/// The memo for a bridged transaction
const BRIDGE_MEMO: [u8; 31] = *b"Bridged from solana via zosh.io";

/// The memo for a change output
const CHANGE_MEMO: [u8; 512] = [0; 512];

impl Light {
    /// Send a fund to a orchard address for development purposes
    pub async fn dev_send(
        &mut self,
        signer: &GroupSigners,
        recipient: UnifiedAddress,
        amount: u64,
    ) -> Result<()> {
        let Some(recipient) = recipient.orchard() else {
            return Err(anyhow::anyhow!("Invalid orchard address"));
        };

        let Some(fvk) = self.ufvk.orchard() else {
            return Err(anyhow::anyhow!("Invalid orchard full viewing key"));
        };

        let spend_value = amount
            .checked_sub(STANDARD_FEE)
            .ok_or_else(|| anyhow::anyhow!("Amount too small to cover transaction fee"))?;

        // 1. get spendable notes - select notes that cover the total amount (spend + fee)
        let (target_height, anchor_height) = self.heights()?;
        let orchard_notes = self.spendable_notes(amount.into(), target_height)?;
        if orchard_notes.is_empty() {
            return Err(anyhow::anyhow!("No spendable notes found"));
        }

        // Calculate total value from all selected notes
        let total_note_value: u64 = orchard_notes
            .iter()
            .map(|note| note.value().into_u64())
            .sum();

        // Calculate change: total_note_value - total_amount (spend + fee)
        let change = total_note_value.checked_sub(amount).ok_or_else(|| {
            anyhow::anyhow!(
                "Total note value {} is less than required amount {}",
                total_note_value,
                amount
            )
        })?;

        // Get anchor and merkle paths for all notes at the anchor height
        let (anchor, merkle_paths) =
            self.wallet
                .with_orchard_tree_mut::<_, _, anyhow::Error>(|tree| {
                    let anchor = tree.root_at_checkpoint_id(&anchor_height)?.ok_or_else(|| {
                        anyhow::anyhow!(
                            "Missing Orchard anchor at height {}",
                            u32::from(anchor_height)
                        )
                    })?;

                    let mut paths = Vec::new();
                    for note in &orchard_notes {
                        let merkle_path = tree
                            .witness_at_checkpoint_id_caching(
                                note.note_commitment_tree_position(),
                                &anchor_height,
                            )?
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "Missing Orchard witness for note {} at height {}",
                                    note.internal_note_id(),
                                    u32::from(anchor_height)
                                )
                            })?;
                        paths.push(merkle_path);
                    }

                    Ok::<_, anyhow::Error>((anchor.into(), paths))
                })?;

        // 2. Prepare outputs: recipient output + change output (if any)
        let mut outputs = Vec::new();

        // Output to recipient with the spend value
        let mut recipient_memo = [0; 512];
        recipient_memo[..31].copy_from_slice(&BRIDGE_MEMO);
        outputs.push(OutputInfo::new(
            None,
            *recipient,
            NoteValue::from_raw(spend_value),
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
            tracing::info!("Sending change of {} zatoshis back to our address", change);
        }

        // 3. Create SpendInfo for all notes being spent
        let mut spend_infos = Vec::new();
        for (note, merkle_path) in orchard_notes.iter().zip(merkle_paths.iter()) {
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
        let utx = TransactionData::<Unauthorized>::from_parts(
            TxVersion::suggested_for_branch(branch_id),
            branch_id,
            0,
            expiry_height,
            None,
            None,
            None,
            Some(bundle),
        );

        // 5. Create proof and prepare for signing
        let tx = signer.sign_tx(utx)?.freeze()?;
        let txid = tx.txid();
        let mut data = Vec::new();
        tx.write(&mut data)?;
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
        Ok(())
    }
}

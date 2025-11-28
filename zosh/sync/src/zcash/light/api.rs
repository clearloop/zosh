//! Light client APIs

use crate::zcash::{
    light::Light,
    signer::{GroupSigners, SignerInfo},
};
use anyhow::Result;
use orchard::{
    builder::{self, BundleType, MaybeSigned, OutputInfo, SpendInfo},
    bundle::Flags,
    circuit::ProvingKey,
    keys::{Scope, SpendValidatingKey},
    primitives::redpallas::{Signature, SpendAuth},
    value::NoteValue,
};
use reddsa::frost::redpallas::Randomizer;
use std::time::Duration;
use zcash_client_backend::{
    data_api::{
        wallet::ConfirmationsPolicy, Account, AccountBirthday, AccountPurpose, InputSource,
        TargetValue, WalletCommitmentTrees, WalletRead, WalletWrite,
    },
    fees::orchard::InputView,
    proto::service::{BlockId, RawTransaction},
    sync,
};
use zcash_keys::{address::UnifiedAddress, keys::UnifiedFullViewingKey};
use zcash_primitives::transaction::{
    sighash::{signature_hash, SignableInput},
    txid::TxIdDigester,
    Authorized, TransactionData, TxVersion, Unauthorized,
};
use zcash_protocol::{
    consensus::{BlockHeight, BranchId},
    value::{ZatBalance, Zatoshis},
    ShieldedProtocol,
};

impl Light {
    /// Sync the wallet
    pub async fn sync(&mut self) -> Result<()> {
        sync::run(
            &mut self.client,
            &self.network,
            &self.block,
            &mut self.wallet,
            100,
        )
        .await
        .map_err(Into::into)
    }

    /// Sync the wallet
    pub async fn sync_forever(&mut self) -> Result<()> {
        loop {
            self.sync().await?;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    /// Import a unified full viewing key
    pub async fn import(&mut self, name: &str, ufvk: UnifiedFullViewingKey) -> Result<()> {
        let birth = BranchId::Nu6_1
            .height_bounds(&self.network)
            .ok_or(anyhow::anyhow!("Invalid network"))?
            .0
            .into();

        let block = self
            .client
            .get_block(BlockId {
                height: birth,
                hash: Default::default(),
            })
            .await?
            .into_inner();

        let tree = self
            .client
            .get_tree_state(BlockId {
                height: birth,
                hash: block.hash,
            })
            .await?
            .into_inner();

        self.wallet.import_account_ufvk(
            name,
            &ufvk,
            &AccountBirthday::from_treestate(tree, None)
                .map_err(|_e| anyhow::anyhow!("Invalid birthday"))?,
            AccountPurpose::ViewOnly,
            None,
        )?;
        Ok(())
    }

    /// Send a fund to a orchard address
    ///
    /// # Arguments
    /// * `amount` - The total amount to spend (spend value + transaction fee) in ZEC
    pub async fn send(
        &mut self,
        signer: &GroupSigners,
        recipient: UnifiedAddress,
        amount: f32,
    ) -> Result<()> {
        let recipient_orchard = recipient
            .orchard()
            .ok_or(anyhow::anyhow!("Invalid orchard address"))?;
        let ufvk = signer.ufvk()?;

        // Convert amount to zatoshis (amount already includes spend value + fee)
        let total_amount = (amount * 100_000_000.0).round() as u64;

        // Standard Orchard transaction fee (1000 zatoshis = 0.00001 ZEC)
        // This is a typical fee for Orchard transactions
        const STANDARD_FEE: u64 = 1000;

        // Calculate the spend value (amount - fee)
        let spend_value = total_amount
            .checked_sub(STANDARD_FEE)
            .ok_or_else(|| anyhow::anyhow!("Amount too small to cover transaction fee"))?;

        let Some(account) = self.wallet.get_account_for_ufvk(&ufvk)? else {
            return Err(anyhow::anyhow!("Account not found by provided ufvk"));
        };

        // Get target and anchor heights using the wallet's built-in method
        // This ensures we use a valid checkpoint that exists in the tree
        let confirmations_policy = ConfirmationsPolicy::default();
        let (target_height, anchor_height) = self
            .wallet
            .get_target_and_anchor_heights(confirmations_policy.trusted())
            .map_err(|e| anyhow::anyhow!("Failed to get target and anchor heights: {:?}", e))?
            .ok_or_else(|| anyhow::anyhow!("Wallet sync required"))?;

        // 1. get spendable notes - select notes that cover the total amount (spend + fee)
        let notes = self.wallet.select_spendable_notes(
            account.id(),
            TargetValue::AtLeast(Zatoshis::from_u64(total_amount)?),
            &[ShieldedProtocol::Orchard],
            target_height,
            confirmations_policy,
            &[],
        )?;

        let Some(note) = notes.orchard().first() else {
            return Err(anyhow::anyhow!("No spendable notes found"));
        };

        // Calculate change: note_value - total_amount (spend + fee)
        let note_value = note.value().into_u64();
        let change = note_value
            .checked_sub(total_amount)
            .ok_or_else(|| anyhow::anyhow!("Note value is less than required amount"))?;

        // Get anchor and merkle path at the anchor height (guaranteed to have a checkpoint)
        let (anchor, merkle_path) =
            self.wallet
                .with_orchard_tree_mut::<_, _, anyhow::Error>(|tree| {
                    let anchor = tree.root_at_checkpoint_id(&anchor_height)?.ok_or_else(|| {
                        anyhow::anyhow!(
                            "Missing Orchard anchor at height {}",
                            u32::from(anchor_height)
                        )
                    })?;

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

                    Ok::<_, anyhow::Error>((anchor.into(), merkle_path))
                })?;

        // 2. Prepare outputs: recipient output + change output (if any)
        let mut outputs = Vec::new();

        // Output to recipient with the spend value
        let mut recipient_memo = [0; 512];
        recipient_memo[..31].copy_from_slice(b"Bridged from solana via zosh.io");
        outputs.push(OutputInfo::new(
            None,
            *recipient_orchard,
            NoteValue::from_raw(spend_value),
            recipient_memo,
        ));

        // If there's change, send it back to our own address
        if change > 0 {
            // Get our own Orchard address for change
            let fvk = ufvk
                .orchard()
                .ok_or(anyhow::anyhow!("Invalid orchard full viewing key"))?;
            let change_address = fvk.address_at(0u64, Scope::External);
            let change_memo = [0; 512];
            outputs.push(OutputInfo::new(
                None,
                change_address,
                NoteValue::from_raw(change),
                change_memo,
            ));
            tracing::info!("Sending change of {} zatoshis back to our address", change);
        }

        // 3. make the bundle of the transaction
        let Some((bundle, _meta)) = builder::bundle::<ZatBalance>(
            rand_core::OsRng,
            anchor,
            BundleType::Transactional {
                flags: Flags::ENABLED,
                bundle_required: false,
            },
            vec![SpendInfo::new(
                ufvk.orchard()
                    .ok_or(anyhow::anyhow!("Invalid orchard full viewing key"))?
                    .clone(),
                *note.note(),
                merkle_path.into(),
            )
            .ok_or(anyhow::anyhow!("Failed to create spend info"))?],
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

        // 4. Create proof and prepare for signing
        let txid_parts = utx.digest(TxIdDigester);
        let sighash = signature_hash(&utx, &SignableInput::Shielded, &txid_parts);
        let proving_key = ProvingKey::build();
        let proven = utx
            .orchard_bundle()
            .cloned()
            .ok_or(anyhow::anyhow!("Failed to get orchard bundle"))?
            .create_proof(&proving_key, rand_core::OsRng)?
            .prepare(rand_core::OsRng, *sighash.as_ref());
        let fvk = ufvk
            .orchard()
            .ok_or(anyhow::anyhow!("Invalid orchard full viewing key"))?;
        let ak: SpendValidatingKey = fvk.clone().into();
        let mut alphas = Vec::new();
        let proven = proven.map_authorization(
            &mut rand_core::OsRng,
            |_rng, _partial, maybe| {
                if let MaybeSigned::SigningMetadata(parts) = &maybe {
                    if parts.ak == ak {
                        alphas.push(parts.alpha);
                    }
                }
                maybe
            },
            |_rng, auth| auth,
        );

        // 5. Sign the transaction
        let mut signatures = Vec::new();
        for alpha in alphas.iter() {
            let randomizer = Randomizer::from_scalar(*alpha);
            let (signature, _) = signer.sign(sighash.as_ref(), &randomizer)?;
            let sigbytes: [u8; 64] = signature
                .serialize()?
                .try_into()
                .map_err(|_e| anyhow::anyhow!("Failed to convert signature to bytes"))?;
            let signature = Signature::<SpendAuth>::from(sigbytes);
            signatures.push(signature);
        }

        let proven = proven
            .append_signatures(&signatures)
            .map_err(|_e| anyhow::anyhow!("Failed to append signatures"))?
            .finalize()
            .map_err(|_e| anyhow::anyhow!("Failed to finalize"))?;

        let tx = TransactionData::<Authorized>::from_parts(
            TxVersion::suggested_for_branch(branch_id),
            branch_id,
            0,
            expiry_height,
            None,
            None,
            None,
            Some(proven),
        );

        let tx = tx.freeze()?;
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

        // Sync the wallet to update its state after sending the transaction
        // This ensures that spent notes are marked as spent and won't be selected again
        // Note: The transaction may not be mined yet, but syncing will update the state
        // once it's confirmed in a block
        self.sync().await?;
        Ok(())
    }
}

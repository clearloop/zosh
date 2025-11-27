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
    keys::SpendValidatingKey,
    primitives::redpallas::{Signature, SpendAuth},
    value::NoteValue,
};
use reddsa::frost::redpallas::Randomizer;
use zcash_client_backend::{
    data_api::{
        wallet::ConfirmationsPolicy, Account, AccountBirthday, AccountPurpose, InputSource,
        TargetValue, WalletCommitmentTrees, WalletRead, WalletWrite,
    },
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
    pub async fn send(
        &mut self,
        signer: GroupSigners,
        recipient: UnifiedAddress,
        amount: f32,
    ) -> Result<()> {
        let recipient = recipient
            .orchard()
            .ok_or(anyhow::anyhow!("Invalid orchard address"))?;
        let ufvk = signer.ufvk()?;
        let amount = (amount * 100_000_000.0).round() as u64;
        let Some(account) = self.wallet.get_account_for_ufvk(&ufvk)? else {
            return Err(anyhow::anyhow!("Account not found by provided ufvk"));
        };

        // Get target and anchor heights using the wallet's built-in method
        // This ensures we use a valid checkpoint that exists in the tree
        let confirmations_policy = ConfirmationsPolicy::new_symmetrical(1.try_into().unwrap());
        let (target_height, anchor_height) = self
            .wallet
            .get_target_and_anchor_heights(confirmations_policy.trusted())
            .map_err(|e| anyhow::anyhow!("Failed to get target and anchor heights: {:?}", e))?
            .ok_or_else(|| anyhow::anyhow!("Wallet sync required"))?;

        // 1. get spendable notes
        let notes = self.wallet.select_spendable_notes(
            account.id(),
            TargetValue::AtLeast(Zatoshis::from_u64(amount)?),
            &[ShieldedProtocol::Orchard],
            target_height,
            confirmations_policy,
            &[],
        )?;

        let Some(note) = notes.orchard().first() else {
            return Err(anyhow::anyhow!("No spendable notes found"));
        };

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

        // 2. make the bundle of the transaction
        let mut memo = [0; 512];
        memo[..20].copy_from_slice(b"Bridged via Zorch.");
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
            vec![OutputInfo::new(
                None,
                *recipient,
                NoteValue::from_raw(amount),
                memo,
            )],
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

        // 3. Create proof and prepare for signing
        let txid_parts = utx.digest(TxIdDigester);
        let sighash = signature_hash(&utx, &SignableInput::Shielded, &txid_parts);
        tracing::info!("proving ...");
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

        // 4. Sign the transaction
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
        tracing::info!("Transaction ID: {}", txid);

        let mut data = Vec::new();
        tx.write(&mut data)?;
        let resp = self
            .client
            .send_transaction(RawTransaction { data, height: 0 })
            .await?
            .into_inner();

        println!("Transaction sent: {:?}", resp);
        Ok(())
    }
}

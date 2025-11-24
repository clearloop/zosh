//! Light client APIs

use crate::{
    light::Light,
    signer::{GroupSigners, SignerInfo},
};
use anyhow::Result;
use orchard::{
    builder::{self, BundleType, OutputInfo, SpendInfo},
    bundle::Flags,
    value::NoteValue,
    Address,
};
use zcash_client_backend::{
    data_api::{
        chain::BlockCache,
        wallet::{ConfirmationsPolicy, TargetHeight},
        Account, AccountBirthday, AccountPurpose, InputSource, TargetValue, WalletCommitmentTrees,
        WalletRead, WalletWrite,
    },
    proto::service::{BlockId, Empty},
    sync,
};
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_primitives::transaction::{TransactionData, TxVersion, Unauthorized};
use zcash_protocol::{
    consensus::BranchId,
    value::{ZatBalance, Zatoshis},
    ShieldedProtocol,
};

impl Light {
    /// Get the light client info
    pub async fn info(&mut self) -> Result<()> {
        let response = self.client.get_lightd_info(Empty {}).await?;
        let info = response.into_inner();
        println!("Light client info: {:?}", info);
        Ok(())
    }

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
        let birth = BranchId::Nu6
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

    /// Get the accounts
    pub fn summary(&self) -> Result<()> {
        let summary = self
            .wallet
            .get_wallet_summary(ConfirmationsPolicy::new_symmetrical(1.try_into().unwrap()))?
            .ok_or(anyhow::anyhow!("Failed to get wallet summary"))?;

        println!("Wallet summary: {:?}", summary);
        Ok(())
    }

    /// Send a fund to a orchard address
    pub async fn send(
        &mut self,
        signer: GroupSigners,

        recipient: Address,
        amount: f32,
    ) -> Result<()> {
        let ufvk = signer.ufvk()?;
        let amount = (amount * 100_000_000.0).round() as u64;
        let Some(account) = self.wallet.get_account_for_ufvk(&ufvk)? else {
            return Err(anyhow::anyhow!("Account not found by provided ufvk"));
        };

        let Some(latest) = self.block.get_tip_height(None)? else {
            return Err(anyhow::anyhow!("Failed to get tip height"));
        };

        // 1. get spendable notes
        let target_height = TargetHeight::from(latest);
        let notes = self.wallet.select_spendable_notes(
            account.id(),
            TargetValue::AtLeast(Zatoshis::from_u64(amount)?),
            &[ShieldedProtocol::Orchard],
            target_height,
            ConfirmationsPolicy::new_symmetrical(1.try_into().unwrap()),
            &[],
        )?;

        let Some(note) = notes.orchard().first() else {
            return Err(anyhow::anyhow!("No spendable notes found"));
        };

        let anchor_height = latest;
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
                note.note().clone(),
                merkle_path.into(),
            )
            .ok_or(anyhow::anyhow!("Failed to create spend info"))?],
            vec![OutputInfo::new(
                None,
                recipient,
                NoteValue::from_raw(amount),
                [0; 512],
            )],
        )?
        else {
            return Err(anyhow::anyhow!("Failed to create bundle"));
        };

        // 3. create the transaction
        let _tx = TransactionData::<Unauthorized>::from_parts(
            TxVersion::suggested_for_branch(BranchId::Nu6),
            BranchId::Nu6,
            0,
            latest + 20,
            None,
            None,
            None,
            Some(bundle),
        );

        // TODO: prove and sign
        Ok(())
    }
}

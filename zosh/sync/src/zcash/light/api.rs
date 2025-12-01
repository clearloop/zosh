//! Light client APIs

use crate::zcash::{light::ZcashClient, CONFIRMATIONS};
use anyhow::Result;
use orchard::Note;
use std::time::Duration;
use zcash_client_backend::{
    data_api::{
        wallet::{ConfirmationsPolicy, TargetHeight},
        Account, AccountBirthday, AccountPurpose, InputSource, MaxSpendMode, TargetValue,
        WalletRead, WalletWrite,
    },
    proto::service::BlockId,
    sync,
    wallet::ReceivedNote,
};
use zcash_client_sqlite::ReceivedNoteId;
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_protocol::{
    consensus::{BlockHeight, BranchId},
    value::Zatoshis,
    ShieldedProtocol,
};

impl ZcashClient {
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

    /// Get the target and anchor heights
    pub fn heights(&self) -> Result<(TargetHeight, BlockHeight)> {
        let confirmations_policy = ConfirmationsPolicy::default();
        let (target_height, anchor_height) = self
            .wallet
            .get_target_and_anchor_heights(confirmations_policy.trusted())
            .map_err(|e| anyhow::anyhow!("Failed to get target and anchor heights: {:?}", e))?
            .ok_or_else(|| anyhow::anyhow!("Wallet sync required"))?;

        Ok((target_height, anchor_height))
    }

    /// Get the spendable notes
    pub fn spendable_notes(
        &self,
        amount: u64,
        target: TargetHeight,
        exclude: &[ReceivedNoteId],
    ) -> Result<Vec<ReceivedNote<ReceivedNoteId, Note>>> {
        let Some(account) = self.wallet.get_account_for_ufvk(&self.ufvk)? else {
            return Err(anyhow::anyhow!("Account not found by provided ufvk"));
        };

        // Get the spendable notes
        let notes = self.wallet.select_spendable_notes(
            account.id(),
            if amount == 0 {
                TargetValue::AllFunds(MaxSpendMode::MaxSpendable)
            } else {
                TargetValue::AtLeast(Zatoshis::from_u64(amount)?)
            },
            &[ShieldedProtocol::Orchard],
            target,
            CONFIRMATIONS,
            exclude,
        )?;

        Ok(notes.orchard().to_vec())
    }
}

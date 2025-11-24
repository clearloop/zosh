//! Light client APIs

use crate::light::Light;
use anyhow::Result;
use zcash_client_backend::{
    data_api::{
        wallet::ConfirmationsPolicy, AccountBirthday, AccountPurpose, WalletRead, WalletWrite,
    },
    proto::service::{BlockId, Empty},
    sync,
};
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_protocol::consensus::BranchId;

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
}

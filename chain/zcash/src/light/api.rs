//! Light client APIs

use crate::light::Light;
use anyhow::Result;
use zcash_client_backend::{
    data_api::{AccountBirthday, AccountPurpose, WalletWrite},
    proto::service::{BlockId, Empty},
    sync,
};
use zcash_keys::keys::UnifiedFullViewingKey;

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
    pub async fn import(
        &mut self,
        name: &str,
        birth: u32,
        ufvk: UnifiedFullViewingKey,
    ) -> Result<()> {
        let block = self
            .client
            .get_block(BlockId {
                height: birth as u64,
                hash: Default::default(),
            })
            .await?
            .into_inner();

        let tree = self
            .client
            .get_tree_state(BlockId {
                height: birth as u64,
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
}

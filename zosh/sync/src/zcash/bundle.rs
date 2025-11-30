//! Bundle interfaces for zcash

use crate::zcash::ZcashClient;
use anyhow::Result;
use zcash_keys::{address::UnifiedAddress, encoding::AddressCodec};
use zcash_primitives::transaction::{TransactionData, Unauthorized};
use zcore::{
    ex::{Bridge, BridgeBundle},
    registry::Chain,
};

impl ZcashClient {
    /// Bundle the bridge transactions
    ///
    /// TODO: support multiple bridges
    pub async fn bundle(
        &mut self,
        bridges: &[Bridge],
    ) -> Result<(BridgeBundle, TransactionData<Unauthorized>)> {
        let bundle = BridgeBundle::new(Chain::Zcash);
        let bridge = bridges[0].clone();
        let recipient =
            UnifiedAddress::decode(&self.network, &String::from_utf8(bridge.recipient.clone())?)
                .map_err(|e| anyhow::anyhow!("Invalid orchard address: {e:?}"))?;

        let utx = self.tx(recipient, bridge.amount)?;
        Ok((bundle, utx))
    }
}

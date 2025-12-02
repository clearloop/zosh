//! Bundle interfaces for zcash

use crate::{zcash::ZcashClient, ChainFormatEncoder};
use anyhow::Result;
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
        let mut bundle = BridgeBundle::new(Chain::Zcash);
        let bridge = bridges[0].clone();
        let recipient = bridge.recipient.zcash_address(&self.network)?;
        let utx = self.tx(recipient, bridge.amount)?;
        bundle.bridge.push(bridge.clone());
        Ok((bundle, utx))
    }
}

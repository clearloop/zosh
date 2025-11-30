//! Validation interfaces for bridge bundles

use crate::Sync;
use anyhow::Result;
use zcore::{
    ex::{Bridge, BridgeBundle, Receipt},
    registry::{Chain, Coin},
};

impl Sync {
    /// Bundle the bridge requests
    ///
    /// We need to sign the bundles after processed.
    pub async fn bundle(
        &mut self,
        bridges: Vec<Bridge>,
    ) -> Result<(Vec<BridgeBundle>, Vec<Receipt>)> {
        let mut sol_bundles = Vec::new();
        let mut zcash_bundles = Vec::new();
        for bridge in bridges {
            match bridge.target {
                Chain::Solana => sol_bundles.push(bridge),
                Chain::Zcash => zcash_bundles.push(bridge),
            }
        }

        let (sol_bundles, sol_receipts) = self.bundle_sol_bridges(sol_bundles).await?;
        Ok((sol_bundles, sol_receipts))
    }

    /// Bundle the bridge requests for solana
    ///
    /// TODO: make this in parallel
    ///
    /// FIXME: we should not have the receipts here, should be placed
    /// in the network layer.
    pub async fn bundle_sol_bridges(
        &mut self,
        bridges: Vec<Bridge>,
    ) -> Result<(Vec<BridgeBundle>, Vec<Receipt>)> {
        let mut bundles = Vec::new();
        let mut receipts = Vec::new();
        for unbundled in bridges.windows(Chain::Solana.max_bundle_size()) {
            let (bundle, transaction) = self.solana.bundle(unbundled).await?;
            let signature = self
                .solana
                .dev_sign_and_send(transaction, &self.dev_solana_mpc)
                .await?;

            // sign the bundles
            for bridge in unbundled {
                receipts.push(Receipt {
                    anchor: bridge.txid.clone(),
                    coin: Coin::Zec,
                    txid: signature.as_array().to_vec(),
                    source: Chain::Zcash,
                    target: Chain::Solana,
                });
            }
            bundles.push(bundle);
        }

        Ok((bundles, receipts))
    }

    /// Bundle the bridge requests for zcash
    ///
    /// TODO: prove then splitting data for passing the package on chain.
    pub async fn bundle_zcash_bridges(
        &mut self,
        bridges: Vec<Bridge>,
    ) -> Result<(Vec<BridgeBundle>, Vec<Receipt>)> {
        let mut bundles = Vec::new();
        let mut receipts = Vec::new();
        for unbundled in bridges.windows(Chain::Zcash.max_bundle_size()) {
            let (bundle, utx) = self.zcash.bundle(unbundled).await?;
            let txid = self
                .zcash
                .dev_sign_and_send(utx, &self.dev_zcash_mpc)
                .await?;

            // sign the bundles
            for bridge in unbundled {
                receipts.push(Receipt {
                    anchor: bridge.txid.clone(),
                    coin: Coin::Zec,
                    txid: txid.as_ref().to_vec(),
                    source: Chain::Solana,
                    target: Chain::Zcash,
                });
            }
            bundles.push(bundle);
        }

        Ok((bundles, receipts))
    }

    /// Validate the bridge request
    ///
    /// depends on the transaction id of the source chain.
    pub async fn validate_bridge(&mut self, _bridge: &Bridge) -> Result<()> {
        Ok(())
    }
}

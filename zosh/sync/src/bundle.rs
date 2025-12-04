//! Validation interfaces for bridge bundles

use crate::{zcash, ChainFormatEncoder, Sync};
use anyhow::Result;
use std::{collections::HashSet, mem};
use zcore::{
    ex::{Bridge, BridgeBundle, Receipt},
    registry::{Chain, Coin},
};

impl Sync {
    /// Bundle the bridge requests
    ///
    /// We need to sign the bundles after processed.
    ///
    /// TODO: make the bundling process in parallel.
    ///
    /// TODO: check the duplicates bridge requests, if it is
    /// even possible?
    pub async fn bundle(
        &mut self,
        bridges: Vec<Bridge>,
    ) -> Result<(Vec<BridgeBundle>, Vec<Receipt>)> {
        let mut bridges: HashSet<Bridge> = bridges.into_iter().collect();
        bridges.extend(mem::take(&mut self.unresolved));
        let mut sol_bundles = Vec::new();
        let mut zcash_bundles = Vec::new();
        for bridge in bridges {
            match bridge.target {
                Chain::Solana => sol_bundles.push(bridge),
                Chain::Zcash => zcash_bundles.push(bridge),
            }
        }

        let mut bundles = Vec::new();
        let mut receipts = Vec::new();
        if !sol_bundles.is_empty() {
            let (sol_bundles, sol_receipts) = self.bundle_sol_bridges(sol_bundles).await?;
            bundles.extend(sol_bundles);
            receipts.extend(sol_receipts);
        }

        if !zcash_bundles.is_empty() {
            let (zcash_bundles, zcash_receipts) = self.bundle_zcash_bridges(zcash_bundles).await?;
            bundles.extend(zcash_bundles);
            receipts.extend(zcash_receipts);
        }

        Ok((bundles, receipts))
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
        for unbundled in bridges.chunks(Chain::Solana.max_bundle_size()) {
            let Ok((bundle, transaction)) = self.solana.bundle(unbundled).await.inspect_err(|e| {
                tracing::error!("Failed to bundle solana bridges: {:?}", e);
            }) else {
                self.unresolved.extend(unbundled.to_vec());
                continue;
            };

            let Ok(signature) = self
                .solana
                .dev_sign_and_send(transaction, &self.dev_solana_mpc)
                .await
                .inspect_err(|e| {
                    tracing::error!("Failed to sign and send solana transaction: {:?}", e);
                })
            else {
                self.unresolved.extend(unbundled.to_vec());
                continue;
            };

            // sign the bundles
            for bridge in unbundled {
                receipts.push(Receipt {
                    anchor: bridge.txid.clone(),
                    coin: Coin::Zec,
                    txid: signature.as_array().to_vec(),
                    source: Chain::Zcash,
                    target: Chain::Solana,
                });

                tracing::info!(
                    "Fulfilled bridge request from Zcash({}) to Solana({})! amount={} recipient={}",
                    bridge.txid.zcash_txid()?.to_string(),
                    signature,
                    bridge.amount,
                    bridge.recipient.solana_address()?
                );
            }

            bundles.push(bundle);
        }

        Ok((bundles, receipts))
    }

    /// Bundle the bridge requests for zcash
    ///
    /// TODO: prove then splitting data for passing the package on chain.
    ///
    /// FIXME: handle the mempool case, we may not have enough available notes
    /// to bundle when we got a lot of redeem requests at the same time, and
    /// there could be double spending problem for the same note.
    ///
    /// We need to subscribing the notes we are using to other nodes in the network
    /// layer as well, to deduplicating the transactions in bundles.
    pub async fn bundle_zcash_bridges(
        &mut self,
        bridges: Vec<Bridge>,
    ) -> Result<(Vec<BridgeBundle>, Vec<Receipt>)> {
        let mut bundles = Vec::new();
        let mut receipts = Vec::new();
        for unbundled in bridges.chunks(Chain::Zcash.max_bundle_size()) {
            let Ok((bundle, utx)) = self.zcash.bundle(unbundled).await else {
                self.unresolved.extend(unbundled.to_vec());
                continue;
            };

            let Ok(txid) = self
                .zcash
                .dev_sign_and_send(utx, &self.dev_zcash_mpc)
                .await
                .inspect_err(|e| {
                    tracing::error!("Failed to sign and send zcash transaction: {:?}", e);
                })
            else {
                self.unresolved.extend(unbundled.to_vec());
                continue;
            };

            // sign the bundles
            for bridge in unbundled {
                receipts.push(Receipt {
                    anchor: bridge.txid.clone(),
                    coin: Coin::Zec,
                    txid: txid.as_ref().to_vec(),
                    source: Chain::Solana,
                    target: Chain::Zcash,
                });

                tracing::info!(
                    "Fulfilled bridge request from Solana({}) to Zcash({})! amount={} recipient={}",
                    bridge.txid.solana_signature()?,
                    txid,
                    bridge.amount,
                    bridge
                        .recipient
                        .zcash_address(&self.zcash.network)?
                        .encode(&zcash::Network::TestNetwork),
                );
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

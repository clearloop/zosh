//! Validation interfaces for bridge bundles

use crate::Sync;
use anyhow::Result;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use zcash_keys::{address::UnifiedAddress, encoding::AddressCodec};
use zcash_protocol::TxId;
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
            tracing::info!("solana bundle size: {:?}", unbundled.len());
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

                tracing::info!(
                    "Fullfilled bridge request from Zcash({}) to Solana({})! amount={} recipient={}",
                    TxId::from_bytes(bridge.txid.clone().try_into().expect("Invalid zcash txid")),
                    signature,
                    bridge.amount,
                    Pubkey::new_from_array(bridge.recipient.clone().try_into().expect("Invalid solana pubkey"))
                );
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

                let sig: [u8; 64] = bridge
                    .txid
                    .clone()
                    .try_into()
                    .expect("Invalid solana signature");
                tracing::info!(
                    "Fullfilled bridge request from Solana({}) to Zcash({})! amount={} recipient={:?}",
                    Signature::from(sig),
                    txid,
                    bridge.amount,
                    UnifiedAddress::decode(&self.zcash.network, &String::from_utf8(bridge.recipient.clone())?).expect("Invalid zcash address")
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

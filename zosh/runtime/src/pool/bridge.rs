//! The bridge requests of the zosh

use anyhow::Result;
use std::collections::BTreeMap;
use zcore::{
    ex::{Bridge, BridgeBundle},
    registry::Chain,
    Hash,
};

/// The bridge requests pool for zosh
#[derive(Default)]
pub struct BridgePool {
    /// The queue of the bridge requests
    queue: Vec<Bridge>,

    /// The in-progress bridge requests, aggregating signatures
    in_progress: BTreeMap<Hash, BridgeBundle>,

    /// The completed bridge requests
    completed: BTreeMap<Chain, Vec<BridgeBundle>>,
}

impl BridgePool {
    /// Add a bridge request to the pool
    pub fn add(&mut self, bridge: Bridge) {
        self.queue.push(bridge);
    }

    /// Complete a bridge bundle
    pub fn complete(&mut self, bundle_hash: Hash, signature: Vec<u8>, threshold: usize) {
        let Some(bundle) = self.in_progress.get_mut(&bundle_hash) else {
            return;
        };

        // push the signature to the bundle
        bundle.signatures.push(signature);
        if bundle.signatures.len() < threshold {
            return;
        }

        // add the bundle to the completed map
        let chain = self.completed.entry(bundle.target).or_default();
        chain.push(bundle.clone());
        self.in_progress.remove(&bundle_hash);
    }

    /// Prepare the bridge requests for aggregation
    pub fn prepare(&mut self, block: Hash) -> Result<()> {
        let mut bundles = BTreeMap::new();
        for bridge in self.queue.drain(..) {
            let bundle = bundles
                .entry(bridge.target)
                .or_insert_with(|| BridgeBundle::new(bridge.target));

            // reset the bundle if it is full
            if bundle.bridge.len() >= bundle.target.max_bundle_size() {
                let hash = bundle.hash(block)?;
                self.in_progress.insert(hash, bundle.clone());
                *bundle = BridgeBundle::new(bridge.target);
            }

            bundle.bridge.push(bridge);
        }

        Ok(())
    }

    /// Pack the completed bridge requests
    pub fn pack(&mut self) -> BTreeMap<Chain, Vec<BridgeBundle>> {
        let packed = self.completed.clone();
        self.completed.clear();
        packed
    }
}

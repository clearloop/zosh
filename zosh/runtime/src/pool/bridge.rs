//! The bridge requests of the zosh

use anyhow::Result;
use std::collections::BTreeMap;
use zcore::{ex::BridgeBundle, Hash};

/// The bridge requests pool for zosh
#[derive(Default)]
pub struct BridgePool {
    /// The threshold for the bridge requests
    pub threshold: usize,

    /// The in-progress bridge requests, aggregating signatures
    in_progress: BTreeMap<Hash, BridgeBundle>,

    /// The completed bridge requests
    completed: BTreeMap<Hash, BridgeBundle>,
}

impl BridgePool {
    /// Create a new bridge requests pool
    pub fn new(threshold: usize) -> Self {
        Self {
            threshold,
            in_progress: BTreeMap::new(),
            completed: BTreeMap::new(),
        }
    }

    /// Queue a bridge bundle
    pub fn queue(&mut self, bundles: Vec<BridgeBundle>) -> Result<()> {
        for bundle in bundles {
            let hash = bundle.hash()?;
            self.in_progress.insert(hash, bundle);
        }
        Ok(())
    }

    /// Complete a bridge bundle
    ///
    /// NOTE: need to just the voting power once we get to PoS.
    pub fn complete(&mut self, bundle_hash: Hash, signature: Vec<u8>) {
        let Some(bundle) = self.in_progress.get_mut(&bundle_hash) else {
            return;
        };

        // push the signature to the bundle
        bundle.signatures.push(signature);
        if bundle.signatures.len() < self.threshold {
            return;
        }

        // add the bundle to the completed map
        self.completed.insert(bundle_hash, bundle.clone());
        self.in_progress.remove(&bundle_hash);
    }

    /// Pack the completed bridge requests
    pub fn pack(&mut self) -> BTreeMap<Hash, BridgeBundle> {
        let packed = self.completed.clone();
        self.completed.clear();
        packed
    }
}

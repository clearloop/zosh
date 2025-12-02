//! The transaction structure of zorch

use crate::Hash;
pub use bridge::{Bridge, BridgeBundle, Receipt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod bridge;

/// The transactions inside of a block
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Extrinsic {
    /// The bridge transactions
    pub bridge: BTreeMap<Hash, BridgeBundle>,

    /// The receipts of the bridge transactions
    pub receipts: Vec<Receipt>,
}

impl Extrinsic {
    /// Get the number of transactions in the extrinsic
    pub fn count(&self) -> usize {
        self.bridge
            .values()
            .map(|bundle| bundle.bridge.len())
            .sum::<usize>()
            + self.receipts.len()
    }

    /// Get the signatures of the extrinsic
    pub fn txs(&self) -> Vec<Vec<u8>> {
        let mut signatures = Vec::new();
        for bundle in self.bridge.values() {
            for bridge in &bundle.bridge {
                signatures.push(bridge.txid.clone());
            }
        }

        for receipt in &self.receipts {
            signatures.push(receipt.txid.clone());
        }

        signatures.sort();
        signatures
    }
}

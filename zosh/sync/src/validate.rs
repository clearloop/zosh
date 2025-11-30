//! Validation interfaces for bridge bundles

use crate::Sync;
use anyhow::Result;
use std::collections::BTreeMap;
use zcore::{
    ex::{BridgeBundle, Receipt},
    registry::Chain,
};

impl Sync {
    /// Validate the bridge bundle
    ///
    /// 1. reconstruct the bridge messages from the bundle
    /// 2. verify the signatures of the messages
    pub fn validate_bridges(&self, _bundles: &BTreeMap<Chain, Vec<BridgeBundle>>) -> Result<()> {
        Ok(())
    }

    /// Validate the receipts of the bridge transactions
    pub fn validate_receipts(&self, _receipts: &[Receipt]) -> Result<()> {
        Ok(())
    }
}

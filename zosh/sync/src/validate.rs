//! Validation interfaces for bridge bundles

use crate::Sync;
use anyhow::Result;
use zcore::ex::{Bridge, BridgeBundle};

impl Sync {
    /// Bundle the bridge requests
    ///
    /// We need to sign the bundles after processed.
    pub async fn bundle(&mut self, bridges: Vec<Bridge>) -> Result<Vec<BridgeBundle>> {
        // TODO: implement the bundling logic
        Ok(Vec::new())
    }

    /// Validate the bridge request
    ///
    /// depends on the transaction id of the source chain.
    pub async fn validate_bridge(&mut self, _bridge: &Bridge) -> Result<()> {
        Ok(())
    }
}

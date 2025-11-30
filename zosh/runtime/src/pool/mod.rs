//! The mempool of the zosh

use anyhow::Result;
use bridge::BridgePool;
use zcore::{ex::Receipt, Extrinsic};

mod bridge;

/// The mempool of the zosh
#[derive(Default)]
pub struct Pool {
    /// The bridge requests pool
    pub bridge: BridgePool,

    /// The receipt requests
    pub receipt: Vec<Receipt>,
}

impl Pool {
    /// Pack the pool into an extrinsic
    pub fn pack(&mut self) -> Result<Extrinsic> {
        let receipts = self.receipt.drain(..).collect();
        let bridge = self.bridge.pack();
        let extrinsic = Extrinsic { bridge, receipts };
        Ok(extrinsic)
    }
}

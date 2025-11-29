//! The mempool of the zosh

use zcore::{
    req::{Bridge, Receipt},
    Extrinsic,
};

/// The mempool of the zosh
#[derive(Default)]
pub struct Pool {
    /// The bridge requests
    pub bridge: Vec<Bridge>,

    /// The receipt requests
    pub receipt: Vec<Receipt>,
}

impl Pool {
    /// Batch extrinsic from the pool
    pub fn extrinsic(&mut self) -> Extrinsic {
        Extrinsic::default()
    }
}

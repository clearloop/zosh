//! The mempool of the zosh

use zcore::{
    ex::{Bridge, Receipt},
    Extrinsic,
};

/// The mempool of the zosh
#[derive(Default)]
pub struct Pool {
    /// The bridge requests
    pub bridge: Vec<Bridge>,

    /// The receipt requests
    pub receipt: Vec<Receipt>,

    /// The extrinsic of the pool
    pub extrinsic: Extrinsic,
}

impl Pool {
    /// Reset the extrinsic of the pool
    pub fn reset_extrinsic(&mut self) {
        self.extrinsic = Extrinsic::default();
    }
}

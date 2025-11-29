//! The mempool of the zosh

use zcore::req::{Bridge, Receipt};

/// The mempool of the zosh
#[derive(Default)]
pub struct Pool {
    /// The bridge requests
    pub bridge: Vec<Bridge>,

    /// The receipt requests
    pub receipt: Vec<Receipt>,
}

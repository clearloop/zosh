//! Zcash related transactions

use serde::{Deserialize, Serialize};

/// the unlock bundle receipt
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnlockBundleReceipt {
    /// The blake2b hash of the raw unlock bundle
    pub hash: [u8; 32],

    /// The transaction ids we processed on zcash
    pub txid: Vec<u8>,
}

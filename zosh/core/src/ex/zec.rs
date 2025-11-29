//! Zcash related transactions

use serde::{Deserialize, Serialize};

/// The unlock bundle of the transaction
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnlockBundle {
    /// Solana signatures of the unlock bundle
    pub source: Vec<Vec<u8>>,
}

/// the unlock bundle receipt
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnlockBundleReceipt {
    /// The blake2b hash of the raw unlock bundle
    pub hash: [u8; 32],

    /// The transaction ids we processed on zcash
    pub txid: Vec<u8>,
}

//! Zcash related transactions

use serde::{Deserialize, Serialize};

/// Spend bundle for unlocking zec
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnlockBundle {
    /// The outputs of the frost wallet
    ///
    /// The raw transaction of zcash
    pub raw: Vec<u8>,

    /// Source signatures from solana
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

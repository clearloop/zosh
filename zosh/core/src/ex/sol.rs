//! Solana related transactions

use crate::Message;
use serde::{Deserialize, Serialize};

/// The receipt of the mint bundle
///
/// This transaction is used to verify the mint bundle has
/// been finalized on the solana side.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintBundleReceipt {
    /// The hash of the mint bundle
    pub hash: [u8; 32],

    /// The signature of the receipt (solana side)
    ///
    /// TODO: handle big array for serde
    pub signature: Vec<u8>,

    /// The nonce of the bundle
    pub nonce: u64,
}

impl Message for MintBundleReceipt {
    fn message(&self) -> Vec<u8> {
        let mut message = self.nonce.to_le_bytes().to_vec();
        message.extend_from_slice(&self.signature.to_vec());
        message
    }
}

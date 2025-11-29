//! Solana related transactions

use serde::{Deserialize, Serialize};

use crate::Message;

/// The bridge bundle of the transaction
///
/// Note: one block can only contain one solana bridge
/// bundle at the moment due to the current design of nonce.
///
/// TODO: we should have max limit of the entries due to
/// to make sure that it can be processed on solana side.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintBundle {
    /// The mint entries
    pub entries: Vec<([u8; 32], u64)>,

    /// The signatures of the bundle
    ///
    /// TODO: handle big array for serde
    pub signatures: Vec<Vec<u8>>,

    /// The source of the bundle (zec side)
    pub source: Vec<Vec<u8>>,
}

impl Message for MintBundle {
    fn message(&self) -> Vec<u8> {
        let mut message = Vec::new();
        for entry in &self.entries {
            message.extend_from_slice(entry.0.as_ref());
            message.extend_from_slice(&entry.1.to_le_bytes());
        }
        message
    }
}

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

    /// The source of the receipt (zec side)
    pub source: Vec<u8>,
}

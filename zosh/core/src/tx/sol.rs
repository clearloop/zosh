//! Solana related transactions

use crate::tx::Message;
use serde::{Deserialize, Serialize};

/// The bridge bundle of the transaction
///
/// Note: one block can only contain one solana bridge
/// bundle at the moment due to the current design of nonce.
///
/// TODO: we should have max limit of the entries due to
/// to make sure that it can be processed on solana side.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MintBundle {
    /// The nonce of the bundle could also to the
    /// uniqued id of this bundle atm.
    pub nonce: u64,

    /// The mint entries
    pub entries: Vec<([u8; 32], u64)>,

    /// The signatures of the bundle
    ///
    /// TODO: handle big array for serde
    pub signatures: Vec<Vec<u8>>,

    /// The source txids of the bundle to verify
    /// the entries are valid. (zcash side)
    ///
    /// TODO: handle big array for serde
    /// TODO: shall we embed the sources to the sol
    /// bundle as well?
    pub source: Vec<Vec<u8>>,
}

impl Message for MintBundle {
    fn message(&self) -> Vec<u8> {
        let mut message = self.nonce.to_le_bytes().to_vec();
        for entry in &self.entries {
            message.extend_from_slice(entry.0.as_ref());
            message.extend_from_slice(&entry.1.to_le_bytes());
        }

        message
    }

    fn append_signature(&mut self, signature: [u8; 64]) {
        self.signatures.push(signature.to_vec())
    }
}

/// The receipt of the mint bundle
///
/// This transaction is used to verify the mint bundle has
/// been finalized on the solana side.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintBundleReceipt {
    /// The signature of the receipt (solana side)
    ///
    /// TODO: handle big array for serde
    pub signature: Vec<u8>,

    /// The nonce of the bundle
    pub nonce: u64,

    /// Validators exceed the threshold should sign the receipt
    /// to make sure this transaction is valid.
    ///
    /// TODO: handle big array for serde
    pub signatures: Vec<Vec<u8>>,
}

impl Message for MintBundleReceipt {
    fn append_signature(&mut self, signature: [u8; 64]) {
        self.signatures.push(signature.to_vec())
    }

    fn message(&self) -> Vec<u8> {
        let mut message = self.nonce.to_le_bytes().to_vec();
        message.extend_from_slice(&self.signature.to_vec());
        message
    }
}

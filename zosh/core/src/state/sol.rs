//! Solana related state

use crate::Message;
use serde::{Deserialize, Serialize};

/// The bridge bundle of the transaction
///
/// Note: one block can only contain one solana bridge
/// bundle at the moment due to the current design of nonce.
///
/// TODO: we should have max limit of the entries due to
/// to make sure that it can be processed on solana side.
#[derive(Serialize, Deserialize, Debug, Clone)]
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
}

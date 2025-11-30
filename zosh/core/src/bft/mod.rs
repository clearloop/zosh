//! zoshBFT related primitives

use crate::{FixedBytes, Header};
use anyhow::Result;
use crypto::ed25519;
use serde::{Deserialize, Serialize};

/// The zoshBFT consensus state
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Bft {
    /// The validators of the BFT
    ///
    /// A set of ed25519 public keys
    pub validators: Vec<[u8; 32]>,

    /// The threshold for the BFT
    ///
    /// The number of validators that need to sign the block
    pub threshold: u8,

    /// The authoring randomness series
    pub series: Vec<[u8; 32]>,
}

impl Bft {
    /// Validate the votes of the block
    pub fn validate_votes(&self, header: &Header) -> Result<()> {
        let hash = header.hash();
        let mut votes = 0;

        // TODO: make this in parallel
        for (key, sig) in header.votes.iter() {
            if !self.validators.contains(key) {
                continue;
            }

            if ed25519::verify(key, &hash, &sig.bytes64()?).is_ok() {
                votes += 1;
            }
        }

        if votes < self.threshold as usize {
            anyhow::bail!(
                "Insufficient votes, expected {} votes, got {}",
                self.threshold,
                votes
            );
        }
        Ok(())
    }
}

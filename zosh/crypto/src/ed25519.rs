//! Ed25519 primitives

use anyhow::Result;
use ed25519_dalek::{Signature, VerifyingKey};

/// Verify the signature of the message
pub fn verify(pk: &[u8; 32], msg: &[u8], sig: &[u8; 64]) -> Result<()> {
    let vk = VerifyingKey::from_bytes(pk)?;
    let sig = Signature::from_bytes(sig);
    vk.verify_strict(msg, &sig).map_err(Into::into)
}

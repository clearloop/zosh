//! Utility traits for the core library

use anyhow::Result;

/// The message trait
pub trait Message {
    /// Get the message need to sign for the transaction
    fn message(&self) -> Vec<u8>;
}

/// Convert an array of bytes to a signature
pub trait ToSig {
    /// Convert the data to a signature
    fn ed25519(&self) -> Result<[u8; 64]>;

    /// Convert the data to a signature
    fn ed25519_unchecked(&self) -> [u8; 64];
}

impl ToSig for &Vec<u8> {
    fn ed25519(&self) -> Result<[u8; 64]> {
        if self.len() != 64 {
            anyhow::bail!(
                "Invalid signature length, expected 64 bytes, got {}",
                self.len()
            );
        }

        let mut sig = [0u8; 64];
        sig.copy_from_slice(&self[..64]);
        Ok(sig)
    }

    fn ed25519_unchecked(&self) -> [u8; 64] {
        let mut sig = [0u8; 64];
        sig.copy_from_slice(&self[..64]);
        sig
    }
}

impl ToSig for Vec<u8> {
    fn ed25519(&self) -> Result<[u8; 64]> {
        (&self).ed25519()
    }

    fn ed25519_unchecked(&self) -> [u8; 64] {
        (&self).ed25519_unchecked()
    }
}

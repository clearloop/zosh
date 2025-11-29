//! Signers for the solana bridge

pub use group::GroupSigners;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{Signer, SignerError},
};

mod group;
mod share;

/// Trait for solana signer information
pub trait SolanaSignerInfo {
    /// Get the public key of the signer
    fn public_key(&self) -> Pubkey;

    /// Get the public key as a signer
    fn public(&self) -> Public {
        Public(self.public_key())
    }
}

/// Public key of a solana signer
pub struct Public(Pubkey);

impl Signer for Public {
    fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
        Ok(self.0)
    }

    fn try_sign_message(&self, _message: &[u8]) -> Result<Signature, SignerError> {
        Err(SignerError::NotEnoughSigners)
    }

    fn is_interactive(&self) -> bool {
        false
    }
}

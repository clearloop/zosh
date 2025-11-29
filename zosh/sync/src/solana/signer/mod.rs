//! Signers for the solana bridge

pub use group::GroupSigners;
use solana_sdk::pubkey::Pubkey;

mod group;
mod share;

/// Trait for solana signer information
pub trait SolanaSignerInfo {
    /// Get the public key of the signer
    fn pubkey(&self) -> Pubkey;
}

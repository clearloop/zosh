//! The transaction structure of zorch

use serde::{Deserialize, Serialize};
pub use {
    sol::{MintBundle, MintBundleReceipt},
    zec::{UnlockBundle, UnlockBundleReceipt},
};

mod sol;
mod zec;

/// The transactions inside of a block
#[derive(Debug, Serialize, Deserialize)]
pub struct Extrinsic {
    /// Solana mint bundle
    ///
    /// FIXME: support multipe bundles after removing
    /// the design of nonce.
    pub mint: Option<MintBundle>,

    /// The receipts of mint bundles, could be async.
    pub mint_receipts: Vec<MintBundleReceipt>,

    /// The unlock bundles
    pub unlock: Vec<UnlockBundle>,

    /// The unlock receipts
    pub unlock_receipts: Vec<UnlockBundleReceipt>,
}

/// The message trait
pub trait Message {
    /// Get the message need to sign for the transaction
    fn message(&self) -> Vec<u8>;
}

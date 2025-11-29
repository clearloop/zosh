//! The transaction structure of zorch

use serde::{Deserialize, Serialize};
pub use sol::{MintBundle, MintBundleReceipt};

mod sol;
mod zec;

/// The transactions inside of a block
#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    /// Solana mint bundle
    ///
    /// FIXME: support multipe bundles after removing
    /// the design of nonce.
    pub sol_mint: Option<MintBundle>,

    /// The receipts of mint bundles, could be async.
    pub sol_mint_receipts: Vec<MintBundleReceipt>,
}

/// The message trait
pub trait Message {
    /// Get the message need to sign for the transaction
    fn message(&self) -> Vec<u8>;

    /// Append the signature to the message
    fn append_signature(&mut self, signature: [u8; 64]);
}

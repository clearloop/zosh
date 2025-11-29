//! The transaction structure of zorch

use serde::{Deserialize, Serialize};
pub use {
    sol::{MintBundle, MintBundleReceipt},
    ticket::Ticket,
    zec::{UnlockBundle, UnlockBundleReceipt},
};

mod sol;
mod ticket;
mod zec;

/// The transactions inside of a block
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Extrinsic {
    /// The tickets for rotating the validators
    pub tickets: Vec<Ticket>,

    /// Solana mint bundle
    pub mint: Vec<MintBundle>,

    /// The receipts of mint bundles, could be async.
    pub mint_receipts: Vec<MintBundleReceipt>,

    /// The unlock bundles
    pub unlock: Vec<UnlockBundle>,

    /// The unlock receipts
    pub unlock_receipts: Vec<UnlockBundleReceipt>,
}

/// The transfer of the transaction
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transfer {
    /// The recipient address
    pub recipient: Vec<u8>,

    /// The amount of the transfer
    pub amount: u64,
}

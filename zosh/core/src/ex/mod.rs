//! The transaction structure of zorch

use crate::{state::sol::MintBundle, Sourced};
use serde::{Deserialize, Serialize};
pub use {sol::MintBundleReceipt, ticket::Ticket, zec::UnlockBundleReceipt};

mod sol;
mod ticket;
mod zec;

/// The transactions inside of a block
#[derive(Debug, Serialize, Deserialize)]
pub struct Extrinsic {
    /// The tickets for rotating the validators
    pub tickets: Vec<Ticket>,

    /// Solana mint bundle
    ///
    /// FIXME: support multiple bundles after removing
    /// the design of nonce.
    pub mint: Option<Sourced<MintBundle>>,

    /// The receipts of mint bundles, could be async.
    pub mint_receipts: Vec<MintBundleReceipt>,

    /// The unlock bundles
    pub unlock: Vec<Sourced<Vec<u8>>>,

    /// The unlock receipts
    pub unlock_receipts: Vec<UnlockBundleReceipt>,
}

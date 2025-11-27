//! The archive of the transaction

use crate::tx::Chain;

/// Trim transaction for trimming the chain state
///
/// After processing this transaction, all outer transactions
/// before the specified height will be marked as archived.
///
/// If there are transactions we failed to process or got missed,
/// move the stale transaction to another state, that the original
/// sender can claim them anyway.
pub struct Sync {
    /// The chain to trim
    pub chain: Chain,

    /// The archive height
    pub height: u32,

    /// The operation to perform
    pub operation: Operation,
}

/// The type of the sync transaction

pub enum Operation {
    /// Archive historical data of the target chain at the specified height
    Archive,

    /// Finalize historical data of the target chain at the specified height
    Finalize,
}

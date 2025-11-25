//! The transaction structure of zyphers

pub use bridge::Bridge;

mod bridge;

/// The transaction structure of zyphers
pub struct Transaction {
    /// The bridge transactions
    pub bridge: Vec<Bridge>,
}

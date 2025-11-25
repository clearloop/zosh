//! The transaction structure of zorch

pub use bridge::Bridge;

mod bridge;

/// The transaction structure of zorch
pub struct Transaction {
    /// The bridge transactions
    pub bridge: Vec<Bridge>,
}

//! The transaction structure of zorch

use serde::{Deserialize, Serialize};
pub use {
    bridge::{Bridge, BridgeBundle, Receipt},
    ticket::Ticket,
};

mod bridge;
mod ticket;

/// The transactions inside of a block
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Extrinsic {
    /// The tickets for rotating the validators
    pub tickets: Vec<Ticket>,

    /// The bridge transactions
    pub bridge: Vec<BridgeBundle>,

    /// The receipts of the bridge transactions
    pub receipts: Vec<Receipt>,
}

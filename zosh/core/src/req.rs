//! Requests from the sync clients
//! The bridge transaction structure

use crate::Chain;
use serde::{Deserialize, Serialize};

/// The transaction structure of zorch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bridge {
    /// The recipient address
    pub recipient: Vec<u8>,

    /// The amount of the transaction
    pub amount: u64,

    /// The source of the transaction
    pub source: Chain,

    /// The target chain of the transaction
    pub target: Chain,

    /// The signature of the transaction
    pub txid: Vec<u8>,
}

/// The confirmation of the bridge transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// The anchor signature of the source transaction
    pub anchor: Vec<u8>,

    /// The signature of the confirmation transaction
    pub signature: Vec<u8>,

    /// The source chain of the transaction
    pub source: Chain,

    /// The target chain of the transaction
    pub target: Chain,
}

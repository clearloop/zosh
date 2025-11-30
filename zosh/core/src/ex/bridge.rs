//! The bridge transactions

use crate::{
    registry::{Chain, Coin},
    Hash,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// The signed bridge transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeBundle {
    /// The target chain of the bundle
    pub target: Chain,

    /// The bridge transactions
    pub bridge: Vec<Bridge>,

    /// The data we need for reconstructing the outer transaction
    pub data: Vec<u8>,

    /// The signatures for the upcoming outer transactions
    pub signatures: Vec<Vec<u8>>,
}

impl BridgeBundle {
    /// Compute the hash of the bridge bundle
    pub fn hash(&self, block: Hash) -> Result<Hash> {
        let mut data = block.to_vec();
        data.extend_from_slice(&postcard::to_allocvec(&self)?);
        Ok(crypto::blake3(&data))
    }
}

impl BridgeBundle {
    /// Create a new bridge bundle
    pub fn new(target: Chain) -> Self {
        Self {
            target,
            bridge: Vec::new(),
            data: Vec::new(),
            signatures: Vec::new(),
        }
    }
}

/// The bridge transaction
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bridge {
    /// The token of the transaction
    pub coin: Coin,

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

    /// The coin of the transaction
    pub coin: Coin,

    /// The signature of the confirmation transaction
    pub txid: Vec<u8>,

    /// The source chain of the transaction
    pub source: Chain,

    /// The target chain of the transaction
    pub target: Chain,
}

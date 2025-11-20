//! Configuration of a ZypherBridge client

use serde::{Deserialize, Serialize};

/// Key configuration of a ZypherBridge client
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    /// zcash orchard shard key
    pub zcash: Frost,

    /// solana keypair
    pub solana: String,
}

/// Frost configuration (redjubjub / orachard)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frost {
    /// frost identifier
    pub identifier: String,

    /// frost public key package
    pub package: String,

    /// frost secret share
    pub share: String,
}

//! Configuration for the sync library

use serde::{Deserialize, Serialize};
use url::Url;

/// Configuration for the sync library
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rpc {
    /// solana RPC address
    pub solana: Url,

    /// zcash RPC address
    pub zcash: Url,
}

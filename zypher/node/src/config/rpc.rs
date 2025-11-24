//! RPC configuration

use serde::{Deserialize, Serialize};
use url::Url;

/// RPC configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Rpc {
    /// solana RPC address
    pub solana: Url,

    /// zcash RPC address
    pub lightwalletd: Url,
}

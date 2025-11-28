//! RPC configuration

use serde::{Deserialize, Serialize};
use url::Url;

/// RPC configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Rpc {
    /// solana RPC address
    pub solana: Url,

    /// solana WS address
    pub solana_ws: Url,

    /// zcash RPC address
    pub lightwalletd: Url,
}

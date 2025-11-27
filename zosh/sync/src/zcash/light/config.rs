//! Zcash light client configuration

use crate::zcash::Network;
use std::path::PathBuf;
use url::Url;

/// Zcash light client configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Cache directory
    pub cache: PathBuf,

    /// Wallet directory
    pub wallet: PathBuf,

    /// Lightwalletd URL
    pub lightwalletd: Url,

    /// Network
    pub network: Network,
}

//! Zcash light client configuration

use crate::zcash::Network;
use std::path::PathBuf;
use url::Url;
use zcash_keys::keys::UnifiedFullViewingKey;

/// Zcash light client configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Cache directory
    pub cache: PathBuf,

    /// Wallet directory
    pub wallet: PathBuf,

    /// Lightwalletd URL
    pub lightwalletd: Url,

    /// Network
    pub network: Network,

    /// The unified full viewing key to import
    pub ufvk: UnifiedFullViewingKey,
}

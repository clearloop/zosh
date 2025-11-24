//! Zcash light client configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

/// Zcash light client configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Cache directory
    pub cache: PathBuf,

    /// Wallet directory
    pub wallet: PathBuf,

    /// Lightwalletd URL
    pub lightwalletd: Url,
}

//! Key configuration

use serde::{Deserialize, Serialize};

/// Key configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    /// zcash key
    pub zcash: String,

    /// solana key
    pub solana: String,
}

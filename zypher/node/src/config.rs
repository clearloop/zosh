//! Configuration for the zyphers node

use serde::{Deserialize, Serialize};
pub use {runtime::config::Key, sync::config::Rpc};

/// Configuration for the zyphers node
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// sync configurations
    pub sync: Rpc,

    /// Key configurations
    pub key: Key,
}

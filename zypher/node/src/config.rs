//! Configuration for the ZypherBridge node

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// sync configurations
    pub sync: sync::config::Rpc,

    /// Key configurations
    pub key: runtime::config::Key,
}

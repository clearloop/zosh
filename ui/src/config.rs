//! Configuration module for the UI web service

use anyhow::Result;
use std::{env, net::SocketAddr, path::PathBuf};

/// Configuration for the UI web service
#[derive(Debug, Clone)]
pub struct Config {
    /// Zosh RPC WebSocket URL
    pub rpc_url: String,

    /// SQLite database file path
    pub db_path: PathBuf,

    /// Web service bind address
    pub listen_addr: SocketAddr,
}

impl Config {
    /// Load configuration from environment variables
    pub fn load() -> Result<Self> {
        let home = dirs::home_dir().ok_or(anyhow::anyhow!("Home directory not found"))?;
        let rpc_url =
            env::var("ZOSH_RPC_URL").unwrap_or_else(|_| "ws://localhost:1439".to_string());
        let db_path = home.join(".cache/zosh/ui.db");
        let listen_addr = env::var("ZOSH_LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:1888".to_string())
            .parse::<SocketAddr>()?;

        Ok(Self {
            rpc_url,
            db_path,
            listen_addr,
        })
    }
}

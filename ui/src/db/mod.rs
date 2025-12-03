//! Database module for storing blocks and transactions
#![allow(clippy::type_complexity)]

mod migration;
mod query;
mod sql;

use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

/// Thread-safe database connection wrapper
#[derive(Clone)]
pub struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
}

/// Query result for a bridge transaction
#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeTransactionResult {
    pub txid: String,
    pub coin: String,
    pub amount: u64,
    pub recipient: String,
    pub source: String,
    pub target: String,
    pub slot: u32,
    pub receipt: Option<ReceiptInfo>,
}

/// Receipt information
#[derive(Debug, Serialize, Deserialize)]
pub struct ReceiptInfo {
    pub anchor: String,
    pub coin: String,
    pub txid: String,
    pub source: String,
    pub target: String,
    pub slot: u32,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    /// Total number of blocks
    pub blocks: u32,
    /// Total number of transactions
    pub txns: u32,
    /// Latest block hash (base58)
    pub head: String,
    /// Latest block slot
    pub slot: u32,
    /// Accumulated ZEC bridged to Solana (in zatoshi)
    pub zec_to_solana: u64,
    /// Accumulated ZEC bridged back to Zcash (in zatoshi)
    pub zozec_to_zcash: u64,
    /// Total number of receipts
    pub receipts: u32,
}

impl Database {
    /// Create a new database connection
    pub fn new(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                anyhow::anyhow!("Failed to create directory {}: {}", parent.display(), e)
            })?;
        }
        let conn = Connection::open(db_path).map_err(|e| {
            anyhow::anyhow!("Failed to open database at {}: {}", db_path.display(), e)
        })?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

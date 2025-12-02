//! Database module for storing blocks and transactions

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use zcore::{Block, Head};

/// Thread-safe database connection wrapper
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

/// Query result for a bridge transaction
#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeTransactionResult {
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
    pub txid: String,
    pub slot: u32,
}

impl Database {
    /// Create a new database connection
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Initialize the database schema
    pub fn init(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Create blocks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                slot INTEGER PRIMARY KEY,
                hash BLOB NOT NULL,
                parent BLOB NOT NULL,
                state BLOB NOT NULL,
                accumulator BLOB NOT NULL,
                extrinsic BLOB NOT NULL,
                votes BLOB NOT NULL
            )",
            [],
        )?;

        // Create bridges table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bridges (
                txid BLOB PRIMARY KEY,
                coin TEXT NOT NULL,
                recipient BLOB NOT NULL,
                amount INTEGER NOT NULL,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                block_slot INTEGER NOT NULL,
                bundle_hash BLOB NOT NULL,
                FOREIGN KEY (block_slot) REFERENCES blocks(slot)
            )",
            [],
        )?;

        // Create receipts table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS receipts (
                txid BLOB PRIMARY KEY,
                anchor BLOB NOT NULL,
                coin TEXT NOT NULL,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                block_slot INTEGER NOT NULL,
                FOREIGN KEY (block_slot) REFERENCES blocks(slot)
            )",
            [],
        )?;

        // Create indexes for efficient querying
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bridges_block_slot ON bridges(block_slot)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_receipts_anchor ON receipts(anchor)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_receipts_block_slot ON receipts(block_slot)",
            [],
        )?;

        // Create query_ids table for mapping query IDs to transaction IDs
        conn.execute(
            "CREATE TABLE IF NOT EXISTS query_ids (
                query_id BLOB PRIMARY KEY,
                tx_id BLOB NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    /// Insert a block and its extrinsic data
    pub fn insert_block(&self, block: &Block) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Serialize votes using postcard (since BTreeMap keys are byte arrays, not JSON-compatible)
        let votes_bytes = postcard::to_allocvec(&block.header.votes)?;

        // Compute block hash
        let hash = block.header.hash();

        // Insert block header
        conn.execute(
            "INSERT OR REPLACE INTO blocks (slot, hash, parent, state, accumulator, extrinsic, votes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                block.header.slot,
                &hash[..],
                &block.header.parent[..],
                &block.header.state[..],
                &block.header.accumulator[..],
                &block.header.extrinsic[..],
                &votes_bytes[..],
            ],
        )?;

        // Insert bridge transactions
        for (bundle_hash, bundle) in &block.extrinsic.bridge {
            for bridge in &bundle.bridge {
                let coin_str = format!("{:?}", bridge.coin);
                let source_str = format!("{:?}", bridge.source);
                let target_str = format!("{:?}", bridge.target);

                conn.execute(
                    "INSERT OR REPLACE INTO bridges 
                     (txid, coin, recipient, amount, source, target, block_slot, bundle_hash)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        &bridge.txid[..],
                        coin_str,
                        &bridge.recipient[..],
                        bridge.amount,
                        source_str,
                        target_str,
                        block.header.slot,
                        &bundle_hash[..],
                    ],
                )?;
            }
        }

        // Insert receipts
        for receipt in &block.extrinsic.receipts {
            let coin_str = format!("{:?}", receipt.coin);
            let source_str = format!("{:?}", receipt.source);
            let target_str = format!("{:?}", receipt.target);

            conn.execute(
                "INSERT OR REPLACE INTO receipts 
                 (txid, anchor, coin, source, target, block_slot)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    &receipt.txid[..],
                    &receipt.anchor[..],
                    coin_str,
                    source_str,
                    target_str,
                    block.header.slot,
                ],
            )?;
        }

        Ok(())
    }

    /// Query a bridge transaction by txid
    pub fn query_bridge_tx(&self, txid: &[u8]) -> Result<Option<BridgeTransactionResult>> {
        let conn = self.conn.lock().unwrap();

        // Query the bridge transaction
        let mut stmt = conn.prepare(
            "SELECT coin, recipient, amount, source, target, block_slot
             FROM bridges
             WHERE txid = ?1",
        )?;

        let result: Option<(String, Vec<u8>, u64, String, String, u32)> = stmt
            .query_row(params![txid], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .optional()?;

        let Some((coin, recipient, amount, source, target, block_slot)) = result else {
            return Ok(None);
        };

        // Query for receipt where anchor matches the txid
        let mut receipt_stmt = conn.prepare(
            "SELECT txid, block_slot
             FROM receipts
             WHERE anchor = ?1",
        )?;

        let receipt: Option<ReceiptInfo> = receipt_stmt
            .query_row(params![txid], |row| {
                let receipt_txid: Vec<u8> = row.get(0)?;
                let receipt_slot: u32 = row.get(1)?;
                Ok(ReceiptInfo {
                    txid: encode_txid(&receipt_txid),
                    slot: receipt_slot,
                })
            })
            .optional()?;

        Ok(Some(BridgeTransactionResult {
            coin,
            amount,
            recipient: encode_recipient(&recipient),
            source,
            target,
            slot: block_slot,
            receipt,
        }))
    }

    /// Insert a query ID and tx ID into the database
    pub fn insert_query_id(&self, query_id: Vec<u8>, tx_id: &[u8]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO query_ids (query_id, tx_id) VALUES (?1, ?2)",
            params![query_id, tx_id],
        )?;
        Ok(())
    }

    /// Get tx_id by query_id
    pub fn get_query_id(&self, query_id: &[u8]) -> Result<Option<Vec<u8>>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT tx_id FROM query_ids WHERE query_id = ?1")?;
        let result: Option<Vec<u8>> = stmt
            .query_row(params![query_id], |row| row.get(0))
            .optional()?;
        Ok(result)
    }

    /// Get the latest block head (highest slot)
    pub fn get_latest_head(&self) -> Result<Option<Head>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT slot, hash FROM blocks ORDER BY slot DESC LIMIT 1")?;
        let result: Option<(u32, Vec<u8>)> = stmt
            .query_row([], |row| Ok((row.get(0)?, row.get(1)?)))
            .optional()?;

        let Some((slot, hash_bytes)) = result else {
            return Ok(None);
        };

        let hash: [u8; 32] = hash_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid hash length in database"))?;
        Ok(Some(Head { slot, hash }))
    }
}

/// Encode txid to appropriate string format based on length
fn encode_txid(txid: &[u8]) -> String {
    match txid.len() {
        32 => hex::encode(txid),                // Zcash
        64 => bs58::encode(txid).into_string(), // Solana
        _ => hex::encode(txid),                 // Fallback to hex
    }
}

/// Encode recipient address
fn encode_recipient(recipient: &[u8]) -> String {
    if recipient.len() == 32 {
        // Likely a Solana address
        bs58::encode(recipient).into_string()
    } else {
        // Try as UTF-8 string (for Zcash unified addresses)
        String::from_utf8(recipient.to_vec()).unwrap_or_else(|_| hex::encode(recipient))
    }
}

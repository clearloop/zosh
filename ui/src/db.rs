//! Database module for storing blocks and transactions
#![allow(clippy::type_complexity)]

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use zcore::{Block, Head};

/// Thread-safe database connection wrapper
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
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
                votes BLOB NOT NULL,
                txns INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        // Create bridges table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bridges (
                txid BLOB PRIMARY KEY,
                hash BLOB NOT NULL,
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

        // Create stats table (single row with id=1)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS stats (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                blocks INTEGER NOT NULL DEFAULT 0,
                txns INTEGER NOT NULL DEFAULT 0,
                head TEXT NOT NULL DEFAULT '',
                slot INTEGER NOT NULL DEFAULT 0,
                zec_to_solana INTEGER NOT NULL DEFAULT 0,
                zozec_to_zcash INTEGER NOT NULL DEFAULT 0,
                receipts INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        // Initialize stats row if not exists
        conn.execute(
            "INSERT OR IGNORE INTO stats (id, blocks, txns, head, slot, zec_to_solana, zozec_to_zcash, receipts)
             VALUES (1, 0, 0, '', 0, 0, 0, 0)",
            [],
        )?;

        Ok(())
    }

    /// Insert a block and its extrinsic data
    pub fn insert_block(&self, block: &Block) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Serialize votes using postcard (since BTreeMap keys are byte arrays, not JSON-compatible)
        let votes_bytes = postcard::to_allocvec(&block.header.votes)?;

        // Compute block hash and tx count
        let hash = block.header.hash();
        let txns = block.extrinsic.count() as u32;

        // Insert block header
        conn.execute(
            "INSERT OR REPLACE INTO blocks (slot, hash, parent, state, accumulator, extrinsic, votes, txns)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                block.header.slot,
                &hash[..],
                &block.header.parent[..],
                &block.header.state[..],
                &block.header.accumulator[..],
                &block.header.extrinsic[..],
                &votes_bytes[..],
                txns,
            ],
        )?;

        // Insert bridge transactions
        for (bundle_hash, bundle) in &block.extrinsic.bridge {
            for bridge in &bundle.bridge {
                let bridge_hash = bridge.hash()?;
                let coin_str = format!("{}", bridge.coin);
                let source_str = format!("{:?}", bridge.source);
                let target_str = format!("{:?}", bridge.target);

                conn.execute(
                    "INSERT OR REPLACE INTO bridges 
                     (txid, hash, coin, recipient, amount, source, target, block_slot, bundle_hash)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        &bridge.txid[..],
                        &bridge_hash[..],
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

        // Calculate bridge amounts for stats update
        let mut zec_to_solana: u64 = 0;
        let mut zozec_to_zcash: u64 = 0;

        for bundle in block.extrinsic.bridge.values() {
            for bridge in &bundle.bridge {
                // Check if this is ZEC and determine direction
                if matches!(bridge.coin, zcore::registry::Coin::Zec) {
                    if matches!(bridge.source, zcore::registry::Chain::Zcash)
                        && matches!(bridge.target, zcore::registry::Chain::Solana)
                    {
                        zec_to_solana += bridge.amount;
                    } else if matches!(bridge.source, zcore::registry::Chain::Solana)
                        && matches!(bridge.target, zcore::registry::Chain::Zcash)
                    {
                        zozec_to_zcash += bridge.amount;
                    }
                }
            }
        }

        let receipts_count = block.extrinsic.receipts.len() as u32;
        let head_base58 = bs58::encode(&hash).into_string();

        // Update stats
        conn.execute(
            "UPDATE stats SET
                blocks = blocks + 1,
                txns = txns + ?1,
                head = ?2,
                slot = ?3,
                zec_to_solana = zec_to_solana + ?4,
                zozec_to_zcash = zozec_to_zcash + ?5,
                receipts = receipts + ?6
             WHERE id = 1",
            params![
                txns,
                head_base58,
                block.header.slot,
                zec_to_solana as i64,
                zozec_to_zcash as i64,
                receipts_count,
            ],
        )?;

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
            "SELECT txid, anchor, coin, source, target, block_slot
             FROM receipts
             WHERE anchor = ?1",
        )?;

        let receipt: Option<ReceiptInfo> = receipt_stmt
            .query_row(params![txid], |row| {
                let receipt_txid: Vec<u8> = row.get(0)?;
                let receipt_anchor: Vec<u8> = row.get(1)?;
                let receipt_coin: String = row.get(2)?;
                let receipt_source: String = row.get(3)?;
                let receipt_target: String = row.get(4)?;
                let receipt_slot: u32 = row.get(5)?;
                Ok(ReceiptInfo {
                    anchor: encode_txid(&receipt_anchor),
                    coin: receipt_coin,
                    txid: encode_txid(&receipt_txid),
                    source: receipt_source,
                    target: receipt_target,
                    slot: receipt_slot,
                })
            })
            .optional()?;

        Ok(Some(BridgeTransactionResult {
            txid: encode_txid(txid),
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

    /// Get a block by slot
    pub fn get_block_by_slot(&self, slot: u32) -> Result<Option<Block>> {
        let conn = self.conn.lock().unwrap();
        self.query_block(&conn, "SELECT slot, hash, parent, state, accumulator, extrinsic, votes FROM blocks WHERE slot = ?1", params![slot])
    }

    /// Get a block by hash
    pub fn get_block_by_hash(&self, hash: &[u8]) -> Result<Option<Block>> {
        let conn = self.conn.lock().unwrap();
        self.query_block(&conn, "SELECT slot, hash, parent, state, accumulator, extrinsic, votes FROM blocks WHERE hash = ?1", params![hash])
    }

    /// Internal helper to query a block
    fn query_block(
        &self,
        conn: &Connection,
        sql: &str,
        params: impl rusqlite::Params,
    ) -> Result<Option<Block>> {
        let mut stmt = conn.prepare(sql)?;
        let result: Option<(u32, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> = stmt
            .query_row(params, |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })
            .optional()?;

        let Some((slot, _hash, parent, state, accumulator, extrinsic, votes_bytes)) = result else {
            return Ok(None);
        };

        // Deserialize votes
        let votes: std::collections::BTreeMap<[u8; 32], Vec<u8>> =
            postcard::from_bytes(&votes_bytes)?;

        // Query bridge transactions for this block
        let mut bridge_stmt = conn.prepare(
            "SELECT txid, coin, recipient, amount, source, target, bundle_hash FROM bridges WHERE block_slot = ?1",
        )?;
        let bridges: Vec<(Vec<u8>, String, Vec<u8>, u64, String, String, Vec<u8>)> = bridge_stmt
            .query_map(params![slot], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Query receipts for this block
        let mut receipt_stmt = conn.prepare(
            "SELECT txid, anchor, coin, source, target FROM receipts WHERE block_slot = ?1",
        )?;
        let receipts: Vec<(Vec<u8>, Vec<u8>, String, String, String)> = receipt_stmt
            .query_map(params![slot], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Reconstruct the block
        use std::collections::BTreeMap;
        use zcore::ex::{Bridge, BridgeBundle, Receipt};

        // Group bridges by bundle_hash
        let mut bridge_map: BTreeMap<[u8; 32], BridgeBundle> = BTreeMap::new();
        for (txid, coin, recipient, amount, source, target, bundle_hash) in bridges {
            let bundle_key: [u8; 32] = bundle_hash
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid bundle hash length"))?;

            let bundle = bridge_map
                .entry(bundle_key)
                .or_insert_with(|| BridgeBundle::new(parse_chain(&target)));

            bundle.bridge.push(Bridge {
                coin: parse_coin(&coin),
                recipient,
                amount,
                source: parse_chain(&source),
                target: parse_chain(&target),
                txid,
            });
        }

        // Convert receipts
        let receipt_list: Vec<Receipt> = receipts
            .into_iter()
            .map(|(txid, anchor, coin, source, target)| Receipt {
                anchor,
                coin: parse_coin(&coin),
                txid,
                source: parse_chain(&source),
                target: parse_chain(&target),
            })
            .collect();

        let header = zcore::Header {
            slot,
            parent: parent
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid parent hash"))?,
            state: state
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid state hash"))?,
            accumulator: accumulator
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid accumulator"))?,
            extrinsic: extrinsic
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid extrinsic hash"))?,
            votes,
        };

        Ok(Some(Block {
            header,
            extrinsic: zcore::Extrinsic {
                bridge: bridge_map,
                receipts: receipt_list,
            },
        }))
    }

    /// Get current stats
    pub fn get_stats(&self) -> Result<Stats> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT blocks, txns, head, slot, zec_to_solana, zozec_to_zcash, receipts FROM stats WHERE id = 1",
        )?;

        let stats = stmt.query_row([], |row| {
            Ok(Stats {
                blocks: row.get(0)?,
                txns: row.get(1)?,
                head: row.get(2)?,
                slot: row.get(3)?,
                zec_to_solana: row.get(4)?,
                zozec_to_zcash: row.get(5)?,
                receipts: row.get(6)?,
            })
        })?;

        Ok(stats)
    }

    /// Get paginated blocks with tx count
    pub fn get_blocks_paged(&self, page: u32, row: u32) -> Result<(Vec<(Head, u32)>, u32)> {
        let conn = self.conn.lock().unwrap();

        // Get total count
        let total: u32 = conn.query_row("SELECT COUNT(*) FROM blocks", [], |row| row.get(0))?;

        // Get paginated blocks
        let offset = page * row;
        let mut stmt = conn
            .prepare("SELECT slot, hash, txns FROM blocks ORDER BY slot DESC LIMIT ?1 OFFSET ?2")?;

        let blocks: Vec<(Head, u32)> = stmt
            .query_map(params![row, offset], |row| {
                let slot: u32 = row.get(0)?;
                let hash_bytes: Vec<u8> = row.get(1)?;
                let txns: u32 = row.get(2)?;
                Ok((slot, hash_bytes, txns))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(slot, hash_bytes, txns)| {
                let hash: [u8; 32] = hash_bytes.try_into().ok()?;
                Some((Head { slot, hash }, txns))
            })
            .collect();

        Ok((blocks, total))
    }

    /// Get paginated bridge transactions with optional receipt info
    pub fn get_bridges_paged(
        &self,
        page: u32,
        row: u32,
    ) -> Result<(Vec<BridgeTransactionResult>, u32)> {
        let conn = self.conn.lock().unwrap();

        // Get total count
        let total: u32 = conn.query_row("SELECT COUNT(*) FROM bridges", [], |row| row.get(0))?;

        // Get paginated bridges with LEFT JOIN to receipts
        let offset = page * row;
        let mut stmt = conn.prepare(
            "SELECT b.txid, b.coin, b.recipient, b.amount, b.source, b.target, b.block_slot,
                    r.txid as receipt_txid, r.anchor as receipt_anchor, r.coin as receipt_coin,
                    r.source as receipt_source, r.target as receipt_target, r.block_slot as receipt_slot
             FROM bridges b
             LEFT JOIN receipts r ON r.anchor = b.txid
             ORDER BY b.block_slot DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let bridges: Vec<BridgeTransactionResult> = stmt
            .query_map(params![row, offset], |row| {
                let txid: Vec<u8> = row.get(0)?;
                let coin: String = row.get(1)?;
                let recipient: Vec<u8> = row.get(2)?;
                let amount: u64 = row.get(3)?;
                let source: String = row.get(4)?;
                let target: String = row.get(5)?;
                let slot: u32 = row.get(6)?;
                let receipt_txid: Option<Vec<u8>> = row.get(7)?;
                let receipt_anchor: Option<Vec<u8>> = row.get(8)?;
                let receipt_coin: Option<String> = row.get(9)?;
                let receipt_source: Option<String> = row.get(10)?;
                let receipt_target: Option<String> = row.get(11)?;
                let receipt_slot: Option<u32> = row.get(12)?;
                Ok((
                    txid,
                    coin,
                    recipient,
                    amount,
                    source,
                    target,
                    slot,
                    receipt_txid,
                    receipt_anchor,
                    receipt_coin,
                    receipt_source,
                    receipt_target,
                    receipt_slot,
                ))
            })?
            .filter_map(|r| r.ok())
            .map(
                |(
                    txid,
                    coin,
                    recipient,
                    amount,
                    source,
                    target,
                    slot,
                    receipt_txid,
                    receipt_anchor,
                    receipt_coin,
                    receipt_source,
                    receipt_target,
                    receipt_slot,
                )| {
                    let receipt = match (
                        receipt_txid,
                        receipt_anchor,
                        receipt_coin,
                        receipt_source,
                        receipt_target,
                        receipt_slot,
                    ) {
                        (
                            Some(rtxid),
                            Some(ranchor),
                            Some(rcoin),
                            Some(rsource),
                            Some(rtarget),
                            Some(rslot),
                        ) => Some(ReceiptInfo {
                            anchor: encode_txid(&ranchor),
                            coin: rcoin,
                            txid: encode_txid(&rtxid),
                            source: rsource,
                            target: rtarget,
                            slot: rslot,
                        }),
                        _ => None,
                    };

                    BridgeTransactionResult {
                        txid: encode_txid(&txid),
                        coin,
                        amount,
                        recipient: encode_recipient(&recipient),
                        source,
                        target,
                        slot,
                        receipt,
                    }
                },
            )
            .collect();

        Ok((bridges, total))
    }
}

/// Parse chain from debug string
fn parse_chain(s: &str) -> zcore::registry::Chain {
    match s {
        "Zcash" => zcore::registry::Chain::Zcash,
        "Solana" => zcore::registry::Chain::Solana,
        _ => zcore::registry::Chain::Zcash, // Default fallback
    }
}

/// Parse coin from debug string
fn parse_coin(s: &str) -> zcore::registry::Coin {
    match s.to_uppercase().as_str() {
        "ZEC" => zcore::registry::Coin::Zec,
        _ => zcore::registry::Coin::Zec, // Default fallback
    }
}

/// Encode txid to appropriate string format based on length
fn encode_txid(txid: &[u8]) -> String {
    match txid.len() {
        32 => {
            // Zcash: DB stores original order, display needs reversed
            let mut bytes = txid.to_vec();
            bytes.reverse();
            hex::encode(bytes)
        }
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

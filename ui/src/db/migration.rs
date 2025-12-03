//! Database schema initialization and migrations

use super::Database;
use anyhow::Result;

impl Database {
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
                slot INTEGER NOT NULL,
                bundle_hash BLOB NOT NULL,
                FOREIGN KEY (slot) REFERENCES blocks(slot)
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
                slot INTEGER NOT NULL,
                FOREIGN KEY (slot) REFERENCES blocks(slot)
            )",
            [],
        )?;

        // Create indexes for efficient querying
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bridges_slot ON bridges(slot)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_receipts_anchor ON receipts(anchor)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_receipts_slot ON receipts(slot)",
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
}

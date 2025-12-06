//! SQL insert operations

use super::Database;
use anyhow::Result;
use rusqlite::params;
use zcore::Block;

impl Database {
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
                     (txid, hash, coin, recipient, amount, source, target, slot, bundle_hash)
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
                 (txid, anchor, coin, source, target, slot)
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

    /// Insert a query ID and tx ID into the database
    pub fn insert_query_id(&self, query_id: Vec<u8>, tx_id: &[u8]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO query_ids (query_id, tx_id) VALUES (?1, ?2)",
            params![query_id, tx_id],
        )?;
        Ok(())
    }
}

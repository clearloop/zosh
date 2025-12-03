//! Database query operations

use super::{BridgeTransactionResult, Database, ReceiptInfo, Stats};
use crate::util::{encode_recipient, encode_txid, parse_chain, parse_coin};
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use zcore::{Block, Head};

impl Database {
    /// Query a bridge transaction by txid
    pub fn query_bridge_tx(&self, txid: &[u8]) -> Result<Option<BridgeTransactionResult>> {
        let conn = self.conn.lock().unwrap();

        // Query the bridge transaction
        let mut stmt = conn.prepare(
            "SELECT coin, recipient, amount, source, target, slot
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

        let Some((coin, recipient, amount, source, target, slot)) = result else {
            return Ok(None);
        };

        // Query for receipt where anchor matches the txid
        let mut receipt_stmt = conn.prepare(
            "SELECT txid, anchor, coin, source, target, slot
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
            slot,
            receipt,
        }))
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
            "SELECT txid, coin, recipient, amount, source, target, bundle_hash FROM bridges WHERE slot = ?1",
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
        let mut receipt_stmt = conn
            .prepare("SELECT txid, anchor, coin, source, target FROM receipts WHERE slot = ?1")?;
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
            "SELECT b.txid, b.coin, b.recipient, b.amount, b.source, b.target, b.slot,
                    r.txid as receipt_txid, r.anchor as receipt_anchor, r.coin as receipt_coin,
                    r.source as receipt_source, r.target as receipt_target, r.slot as receipt_slot
             FROM bridges b
             LEFT JOIN receipts r ON r.anchor = b.txid
             ORDER BY b.slot DESC
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
                            Some(txid),
                            Some(anchor),
                            Some(coin),
                            Some(source),
                            Some(target),
                            Some(slot),
                        ) => Some(ReceiptInfo {
                            anchor: encode_txid(&anchor),
                            coin,
                            txid: encode_txid(&txid),
                            source,
                            target,
                            slot,
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

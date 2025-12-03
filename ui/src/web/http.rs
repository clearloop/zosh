//! HTTP REST API handlers

use super::{
    build_tx_response, decode_query_id, decode_txid, query_by_qid, query_by_txid, AppState,
};
use crate::{
    db::{Database, Stats},
    ui::{UIBlock, UIBlocksPage, UIHead, UIReceipt, UITxn, UITxnsPage},
    AppError,
};
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

// === Inner functions for unified handlers ===

/// HTTP handler logic for /tx/{txid}
pub fn get_transaction_inner(db: &Database, txid_str: &str) -> Result<Json<UITxn>, AppError> {
    let (txid_bytes, chain_type) = decode_txid(txid_str).map_err(AppError::BadRequest)?;
    tracing::trace!("HTTP: Querying {} transaction: {}", chain_type, txid_str);
    match query_by_txid(db, &txid_bytes).map_err(AppError::Internal)? {
        Some(tx) => Ok(Json(build_tx_response(tx))),
        None => Err(AppError::NotFound("Transaction not found".to_string())),
    }
}

/// HTTP handler logic for /query/{qid}
pub fn get_query_inner(db: &Database, qid_str: &str) -> Result<Json<Value>, AppError> {
    let qid_bytes = decode_query_id(qid_str).map_err(AppError::BadRequest)?;
    tracing::trace!("HTTP: Querying query_id: {}", qid_str);
    match query_by_qid(db, &qid_bytes).map_err(AppError::Internal)? {
        Some((txid_hex, _tx)) => Ok(Json(json!({ "txid": txid_hex }))),
        None => Err(AppError::NotFound("Query ID not found".to_string())),
    }
}

/// HTTP handler logic for /stats
pub fn get_stats_inner(db: &Database) -> Result<Json<Stats>, AppError> {
    let stats = db
        .get_stats()
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    Ok(Json(stats))
}

/// Handler for GET /latest
pub async fn get_latest(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let result = state
        .db
        .get_latest_head()
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    match result {
        Some(head) => {
            let response = json!({
                "slot": head.slot,
                "hash": bs58::encode(head.hash).into_string(),
            });
            Ok(Json(response))
        }
        None => Err(AppError::NotFound("No blocks found".to_string())),
    }
}

/// Handler for GET /block/{hash_or_slot}
pub async fn get_block(
    State(state): State<AppState>,
    Path(hash_or_slot): Path<String>,
) -> Result<Json<UIBlock>, AppError> {
    // Try to parse as slot number first
    let block = if let Ok(slot) = hash_or_slot.parse::<u32>() {
        state
            .db
            .get_block_by_slot(slot)
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
    } else {
        // Try to decode as base58 hash
        let hash_bytes = bs58::decode(&hash_or_slot).into_vec().map_err(|_| {
            AppError::BadRequest("Invalid hash: must be base58 encoded".to_string())
        })?;

        if hash_bytes.len() != 32 {
            return Err(AppError::BadRequest(format!(
                "Invalid hash length: expected 32 bytes, got {}",
                hash_bytes.len()
            )));
        }

        state
            .db
            .get_block_by_hash(&hash_bytes)
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
    };

    match block {
        Some(b) => Ok(Json(UIBlock::from_block(&b))),
        None => Err(AppError::NotFound("Block not found".to_string())),
    }
}

/// Query parameters for /blocks endpoint
#[derive(Deserialize)]
pub struct BlocksQuery {
    page: Option<u32>,
    row: Option<u32>,
}

/// Handler for GET /blocks?page={page}&row={row}
pub async fn get_blocks(
    State(state): State<AppState>,
    Query(query): Query<BlocksQuery>,
) -> Result<Json<UIBlocksPage>, AppError> {
    let page = query.page.unwrap_or(0);
    let row = query.row.unwrap_or(10).min(100); // Cap at 100 rows per page

    let (blocks, total) = state
        .db
        .get_blocks_paged(page, row)
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let ui_blocks: Vec<UIHead> = blocks
        .into_iter()
        .map(|(head, txns)| UIHead {
            slot: head.slot,
            hash: bs58::encode(head.hash).into_string(),
            txns,
        })
        .collect();

    Ok(Json(UIBlocksPage {
        blocks: ui_blocks,
        total,
        page,
        row,
    }))
}

/// Query parameters for /txns endpoint
#[derive(Deserialize)]
pub struct TxnsQuery {
    page: Option<u32>,
    row: Option<u32>,
}

/// Handler for GET /txns?page={page}&row={row}
pub async fn get_txns(
    State(state): State<AppState>,
    Query(query): Query<TxnsQuery>,
) -> Result<Json<UITxnsPage>, AppError> {
    let page = query.page.unwrap_or(0);
    let row = query.row.unwrap_or(10).min(100); // Cap at 100 rows per page

    let (bridges, total) = state
        .db
        .get_bridges_paged(page, row)
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let ui_txns: Vec<UITxn> = bridges
        .into_iter()
        .map(|tx| UITxn {
            txid: tx.txid,
            coin: tx.coin,
            amount: tx.amount,
            recipient: tx.recipient,
            source: tx.source,
            target: tx.target,
            slot: tx.slot,
            receipt: tx.receipt.map(|r| UIReceipt {
                anchor: r.anchor,
                coin: r.coin,
                txid: r.txid,
                source: r.source,
                target: r.target,
            }),
        })
        .collect();

    Ok(Json(UITxnsPage {
        txns: ui_txns,
        total,
        page,
        row,
    }))
}

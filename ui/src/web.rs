//! Web service module

use crate::{
    db::{Database, Stats},
    ui::{UIBlock, UIBlocksPage, UIHead},
    util, AppError,
};
use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

/// Start the web service
pub async fn serve(listen_addr: SocketAddr, db: Database) -> anyhow::Result<()> {
    let state = AppState { db };

    // Configure CORS to allow requests from any origin
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/tx/{txid}", get(get_transaction))
        .route("/query/{qid}", get(get_query))
        .route("/latest", get(get_latest))
        .route("/block/{hash_or_slot}", get(get_block))
        .route("/blocks", get(get_blocks))
        .route("/stats", get(get_stats))
        .layer(cors)
        .with_state(state);

    tracing::info!("Starting web server on {}", listen_addr);
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Handler for GET /txid/:txid
async fn get_transaction(
    State(state): State<AppState>,
    Path(txid_str): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let txid_bytes = util::decode_txid(&txid_str)?;
    let chain_type = match txid_bytes.len() {
        32 => "Zcash",
        64 => "Solana",
        _ => {
            return Err(AppError::BadRequest(format!(
                "Invalid txid length: expected 32 (Zcash) or 64 (Solana) bytes, got {}",
                txid_bytes.len()
            )));
        }
    };

    tracing::trace!("Querying {} transaction: {}", chain_type, txid_str);

    // Query the database
    let result = state
        .db
        .query_bridge_tx(&txid_bytes)
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    match result {
        Some(tx) => {
            let response = json!({
                "coin": tx.coin,
                "amount": tx.amount,
                "recipient": tx.recipient,
                "source": tx.source,
                "target": tx.target,
                "slot": tx.slot,
                "receipt": tx.receipt,
            });
            Ok(Json(response))
        }
        None => Err(AppError::NotFound("Transaction not found".to_string())),
    }
}

/// Handler for GET /query/:qid
async fn get_query(
    State(state): State<AppState>,
    Path(qid_str): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let qid_bytes = bs58::decode(&qid_str).into_vec().map_err(|_| {
        AppError::BadRequest("Invalid query ID: must be base58 encoded".to_string())
    })?;
    tracing::trace!("Querying query_id: {}", qid_str);
    let result = state
        .db
        .get_query_id(&qid_bytes)
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    match result {
        Some(mut tx_id) => {
            tx_id.reverse();
            let txid_str = hex::encode(&tx_id);
            let response = json!({
                "txid": txid_str,
            });
            Ok(Json(response))
        }
        None => Err(AppError::NotFound("Query ID not found".to_string())),
    }
}

/// Handler for GET /latest
async fn get_latest(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
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

/// Handler for GET /block/:hash_or_slot
async fn get_block(
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
struct BlocksQuery {
    page: Option<u32>,
    row: Option<u32>,
}

/// Handler for GET /blocks?page={page}&row={row}
async fn get_blocks(
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

/// Handler for GET /stats
async fn get_stats(State(state): State<AppState>) -> Result<Json<Stats>, AppError> {
    let stats = state
        .db
        .get_stats()
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    Ok(Json(stats))
}

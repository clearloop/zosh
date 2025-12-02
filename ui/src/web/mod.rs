//! Web service module with shared logic for HTTP and WebSocket handlers

pub mod http;
pub mod ws;

use crate::db::{BridgeTransactionResult, Database, Stats};
use axum::{routing::get, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub stats_tx: broadcast::Sender<Stats>,
}

/// Start the web service
pub async fn serve(
    listen_addr: SocketAddr,
    db: Database,
    stats_tx: broadcast::Sender<Stats>,
) -> anyhow::Result<()> {
    let state = AppState {
        db,
        stats_tx: stats_tx.clone(),
    };

    // Configure CORS to allow requests from any origin
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // WebSocket routes
    let ws_routes = Router::new()
        .route("/stats", get(ws::ws_stats))
        .route("/query/{qid}", get(ws::ws_query))
        .route("/tx/{txid}", get(ws::ws_tx));

    // REST routes
    let app = Router::new()
        .route("/tx/{txid}", get(http::get_transaction))
        .route("/query/{qid}", get(http::get_query))
        .route("/latest", get(http::get_latest))
        .route("/block/{hash_or_slot}", get(http::get_block))
        .route("/blocks", get(http::get_blocks))
        .route("/stats", get(http::get_stats))
        .nest("/ws", ws_routes)
        .layer(cors)
        .with_state(state);

    tracing::info!("Starting web server on {}", listen_addr);
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// === Shared query logic ===

/// Decode and validate a query ID (base58 encoded)
pub fn decode_query_id(qid_str: &str) -> Result<Vec<u8>, String> {
    bs58::decode(qid_str)
        .into_vec()
        .map_err(|_| "Invalid query ID: must be base58 encoded".to_string())
}

/// Decode and validate a transaction ID (hex for Zcash, base58 for Solana)
pub fn decode_txid(txid_str: &str) -> Result<(Vec<u8>, &'static str), String> {
    let txid_bytes = crate::util::decode_txid(txid_str)
        .map_err(|_| "Invalid txid format: must be hex or base58 encoded".to_string())?;

    let chain_type = match txid_bytes.len() {
        32 => "Zcash",
        64 => "Solana",
        _ => {
            return Err(format!(
                "Invalid txid length: expected 32 (Zcash) or 64 (Solana) bytes, got {}",
                txid_bytes.len()
            ));
        }
    };

    Ok((txid_bytes, chain_type))
}

/// Build a JSON response for a bridge transaction
pub fn build_tx_response(tx: &BridgeTransactionResult) -> Value {
    json!({
        "coin": tx.coin,
        "amount": tx.amount,
        "recipient": tx.recipient,
        "source": tx.source,
        "target": tx.target,
        "slot": tx.slot,
        "receipt": tx.receipt,
    })
}

/// Query a transaction by query ID, returns (txid_hex, tx_info) if found
pub fn query_by_qid(
    db: &Database,
    qid_bytes: &[u8],
) -> Result<Option<(String, BridgeTransactionResult)>, String> {
    match db.get_query_id(qid_bytes) {
        Ok(Some(mut tx_id)) => {
            tx_id.reverse();
            let txid_hex = hex::encode(&tx_id);
            match db.query_bridge_tx(&tx_id) {
                Ok(Some(tx)) => Ok(Some((txid_hex, tx))),
                Ok(None) => Ok(None),
                Err(e) => Err(format!("Database error: {}", e)),
            }
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Query a transaction by txid
pub fn query_by_txid(
    db: &Database,
    txid_bytes: &[u8],
) -> Result<Option<BridgeTransactionResult>, String> {
    db.query_bridge_tx(txid_bytes)
        .map_err(|e| format!("Database error: {}", e))
}

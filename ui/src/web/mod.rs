//! Web service module with unified HTTP and WebSocket handlers
//!
//! All endpoints support both HTTP and WebSocket:
//! - HTTP: Returns immediately (data or 404)
//! - WebSocket: Waits for data if not found, streams updates for stats
//!
//! Server runs on single address (e.g., http://localhost:1439 / ws://localhost:1439)

pub mod http;
pub mod ws;

use crate::db::{BridgeTransactionResult, Database, Stats};
use axum::{
    extract::{ws::WebSocketUpgrade, FromRequest, Path, Request, State},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::{net::SocketAddr, time::Duration};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

/// Stats polling interval
const STATS_POLL_INTERVAL: Duration = Duration::from_secs(3);

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
        db: db.clone(),
        stats_tx: stats_tx.clone(),
    };

    // Spawn background task to poll stats and broadcast changes
    let poll_db = db.clone();
    let poll_tx = stats_tx.clone();
    tokio::spawn(async move {
        stats_polling_task(poll_db, poll_tx).await;
    });

    // Configure CORS to allow requests from any origin
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Unified routes - same endpoints for HTTP and WebSocket
    // Handlers check for WebSocket upgrade header and handle accordingly
    let app = Router::new()
        .route("/tx/{txid}", get(handle_transaction))
        .route("/query/{qid}", get(handle_query))
        .route("/stats", get(handle_stats))
        .route("/latest", get(http::get_latest))
        .route("/block/{hash_or_slot}", get(http::get_block))
        .route("/blocks", get(http::get_blocks))
        .route("/txns", get(http::get_txns))
        .layer(cors)
        .with_state(state);

    tracing::info!("Starting web server on {}", listen_addr);
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Background task that polls stats every 3 seconds and broadcasts changes
async fn stats_polling_task(db: Database, stats_tx: broadcast::Sender<Stats>) {
    let mut last_stats: Option<Stats> = None;

    loop {
        tokio::time::sleep(STATS_POLL_INTERVAL).await;

        if let Ok(stats) = db.get_stats() {
            // Only broadcast if stats changed
            let should_broadcast = match &last_stats {
                Some(last) => {
                    last.blocks != stats.blocks
                        || last.txns != stats.txns
                        || last.slot != stats.slot
                        || last.receipts != stats.receipts
                }
                None => true,
            };

            if should_broadcast {
                // Ignore send errors (no subscribers)
                let _ = stats_tx.send(stats.clone());
                last_stats = Some(stats);
            }
        }
    }
}

// === Unified Handlers (HTTP + WebSocket) ===

/// Check if request is a WebSocket upgrade
fn is_websocket_request(req: &Request) -> bool {
    req.headers()
        .get(axum::http::header::UPGRADE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}

/// Unified handler for /tx/{txid}
/// - WebSocket: Waits until transaction is found
/// - HTTP: Returns immediately (200 with data or 404)
pub async fn handle_transaction(
    State(state): State<AppState>,
    Path(txid_str): Path<String>,
    req: Request,
) -> Response {
    if is_websocket_request(&req) {
        // WebSocket: upgrade and handle subscription
        match WebSocketUpgrade::from_request(req, &state).await {
            Ok(ws) => {
                ws.on_upgrade(move |socket| ws::handle_tx_subscription(socket, state, txid_str))
            }
            Err(e) => e.into_response(),
        }
    } else {
        // HTTP: immediate response
        http::get_transaction_inner(&state.db, &txid_str).into_response()
    }
}

/// Unified handler for /query/{qid}
/// - WebSocket: Waits until query ID resolves to transaction
/// - HTTP: Returns immediately (200 with txid or 404)
pub async fn handle_query(
    State(state): State<AppState>,
    Path(qid_str): Path<String>,
    req: Request,
) -> Response {
    if is_websocket_request(&req) {
        // WebSocket: upgrade and handle subscription
        match WebSocketUpgrade::from_request(req, &state).await {
            Ok(ws) => {
                ws.on_upgrade(move |socket| ws::handle_query_subscription(socket, state, qid_str))
            }
            Err(e) => e.into_response(),
        }
    } else {
        // HTTP: immediate response
        http::get_query_inner(&state.db, &qid_str).into_response()
    }
}

/// Unified handler for /stats
/// - WebSocket: Streams stats updates as they change
/// - HTTP: Returns current stats
pub async fn handle_stats(State(state): State<AppState>, req: Request) -> Response {
    if is_websocket_request(&req) {
        // WebSocket: upgrade and handle subscription
        match WebSocketUpgrade::from_request(req, &state).await {
            Ok(ws) => ws.on_upgrade(move |socket| ws::handle_stats_subscription(socket, state)),
            Err(e) => e.into_response(),
        }
    } else {
        // HTTP: immediate response
        http::get_stats_inner(&state.db).into_response()
    }
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
        "txid": tx.txid,
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
        Ok(Some(tx_id)) => {
            // tx_id is in original order, reverse for display
            let mut display_bytes = tx_id.clone();
            display_bytes.reverse();
            let txid_hex = hex::encode(&display_bytes);
            // Query DB with original order (tx_id as-is)
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

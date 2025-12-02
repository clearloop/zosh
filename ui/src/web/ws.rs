//! WebSocket subscription handlers

use super::{
    build_tx_response, decode_query_id, decode_txid, query_by_qid, query_by_txid, AppState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::SinkExt;
use serde_json::json;
use std::time::Duration;
use tokio::sync::broadcast;

/// Polling interval for bridge subscriptions
const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Helper to create a text message
fn text_msg(s: impl Into<String>) -> Message {
    Message::Text(s.into().into())
}

/// WebSocket handler for `/ws/stats`
///
/// Subscribes to stats updates and sends them to the client on each block import.
pub async fn ws_stats(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_stats_subscription(socket, state))
}

async fn handle_stats_subscription(mut socket: WebSocket, state: AppState) {
    // Send current stats immediately
    if let Ok(stats) = state.db.get_stats() {
        let msg = serde_json::to_string(&stats).unwrap_or_default();
        if socket.send(text_msg(msg)).await.is_err() {
            return;
        }
    }

    // Subscribe to stats broadcast channel
    let mut rx = state.stats_tx.subscribe();

    loop {
        tokio::select! {
            // Receive stats update from broadcast channel
            result = rx.recv() => {
                match result {
                    Ok(stats) => {
                        let msg = serde_json::to_string(&stats).unwrap_or_default();
                        if socket.send(text_msg(msg)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Skip lagged messages
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
            // Handle incoming messages (ping/pong, close)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// WebSocket handler for `/ws/query/{qid}`
///
/// Polls for query ID resolution until the transaction ID is found,
/// then returns the full transaction info.
pub async fn ws_query(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(qid_str): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_query_subscription(socket, state, qid_str))
}

async fn handle_query_subscription(mut socket: WebSocket, state: AppState, qid_str: String) {
    // Decode query ID
    let qid_bytes = match decode_query_id(&qid_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            let _ = socket
                .send(text_msg(json!({ "error": e }).to_string()))
                .await;
            let _ = socket.close().await;
            return;
        }
    };

    tracing::trace!("WebSocket subscription for query_id: {}", qid_str);

    // Poll until we find the transaction
    loop {
        match query_by_qid(&state.db, &qid_bytes) {
            Ok(Some((txid_hex, tx))) => {
                // Found the transaction
                let mut response = build_tx_response(&tx);
                response["status"] = json!("found");
                response["txid"] = json!(txid_hex);
                let _ = socket.send(text_msg(response.to_string())).await;
                let _ = socket.close().await;
                return;
            }
            Ok(None) => {
                // Check if query ID is resolved but tx not yet in bridges
                if let Ok(Some(mut tx_id)) = state.db.get_query_id(&qid_bytes) {
                    tx_id.reverse();
                    let response = json!({
                        "status": "pending",
                        "txid": hex::encode(&tx_id),
                        "message": "Transaction ID resolved, waiting for bridge confirmation"
                    });
                    let _ = socket.send(text_msg(response.to_string())).await;
                } else {
                    // Not found yet, send pending status
                    let response = json!({
                        "status": "pending",
                        "message": "Waiting for query ID resolution"
                    });
                    let _ = socket.send(text_msg(response.to_string())).await;
                }
            }
            Err(e) => {
                let _ = socket
                    .send(text_msg(json!({ "error": e }).to_string()))
                    .await;
                let _ = socket.close().await;
                return;
            }
        }

        // Check for client disconnect
        tokio::select! {
            _ = tokio::time::sleep(POLL_INTERVAL) => {}
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => return,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            return;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// WebSocket handler for `/ws/tx/{txid}`
///
/// Polls for transaction info until found, then returns it.
pub async fn ws_tx(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(txid_str): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_tx_subscription(socket, state, txid_str))
}

async fn handle_tx_subscription(mut socket: WebSocket, state: AppState, txid_str: String) {
    // Decode transaction ID
    let (txid_bytes, chain_type) = match decode_txid(&txid_str) {
        Ok(result) => result,
        Err(e) => {
            let _ = socket
                .send(text_msg(json!({ "error": e }).to_string()))
                .await;
            let _ = socket.close().await;
            return;
        }
    };

    tracing::trace!(
        "WebSocket subscription for {} transaction: {}",
        chain_type,
        txid_str
    );

    // Poll until we find the transaction
    loop {
        match query_by_txid(&state.db, &txid_bytes) {
            Ok(Some(tx)) => {
                let mut response = build_tx_response(&tx);
                response["status"] = json!("found");
                let _ = socket.send(text_msg(response.to_string())).await;
                let _ = socket.close().await;
                return;
            }
            Ok(None) => {
                // Not found yet, send pending status
                let response = json!({
                    "status": "pending",
                    "message": format!("Waiting for {} transaction", chain_type)
                });
                let _ = socket.send(text_msg(response.to_string())).await;
            }
            Err(e) => {
                let _ = socket
                    .send(text_msg(json!({ "error": e }).to_string()))
                    .await;
                let _ = socket.close().await;
                return;
            }
        }

        // Check for client disconnect
        tokio::select! {
            _ = tokio::time::sleep(POLL_INTERVAL) => {}
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => return,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            return;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

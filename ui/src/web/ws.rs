//! WebSocket subscription handlers

use super::{
    build_tx_response, decode_query_id, decode_txid, query_by_qid, query_by_txid, AppState,
};
use crate::ui::UITxn;
use axum::extract::ws::{Message, WebSocket};
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

/// Handle WebSocket subscription for stats updates
pub async fn handle_stats_subscription(mut socket: WebSocket, state: AppState) {
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

/// Handle WebSocket subscription for query ID resolution
pub async fn handle_query_subscription(socket: WebSocket, state: AppState, qid_str: String) {
    let qid_bytes = match decode_query_id(&qid_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            let mut socket = socket;
            let _ = socket
                .send(text_msg(json!({ "error": e }).to_string()))
                .await;
            let _ = socket.close().await;
            return;
        }
    };

    tracing::trace!("WebSocket subscription for query_id: {}", qid_str);
    poll_until_found(socket, || match query_by_qid(&state.db, &qid_bytes) {
        Ok(Some((_txid_hex, tx))) => PollResult::Found(build_tx_response(tx)),
        Ok(None) => PollResult::NotFound,
        Err(e) => PollResult::Error(e),
    })
    .await;
}

/// Handle WebSocket subscription for transaction lookup
pub async fn handle_tx_subscription(socket: WebSocket, state: AppState, txid_str: String) {
    let (txid_bytes, chain_type) = match decode_txid(&txid_str) {
        Ok(result) => result,
        Err(e) => {
            let mut socket = socket;
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

    poll_until_found(socket, || match query_by_txid(&state.db, &txid_bytes) {
        Ok(Some(tx)) => PollResult::Found(build_tx_response(tx)),
        Ok(None) => PollResult::NotFound,
        Err(e) => PollResult::Error(e),
    })
    .await;
}

/// Result of a poll operation
enum PollResult {
    /// Data found
    Found(UITxn),
    /// Not found yet
    NotFound,
    /// Error occurred
    Error(String),
}

/// Poll until data is found, sending pending message once at start
async fn poll_until_found<F>(mut socket: WebSocket, mut poll_fn: F)
where
    F: FnMut() -> PollResult,
{
    let mut sent = false;
    // Poll until we find data
    loop {
        match poll_fn() {
            PollResult::Found(response) => {
                let msg = serde_json::to_string(&response).unwrap_or_default();
                let _ = socket.send(text_msg(msg)).await;
                let _ = socket.close().await;
                return;
            }
            PollResult::NotFound => {
                if !sent {
                    let _ = socket
                        .send(text_msg(json!({ "status": "pending" }).to_string()))
                        .await;
                    sent = true;
                }
            }
            PollResult::Error(e) => {
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

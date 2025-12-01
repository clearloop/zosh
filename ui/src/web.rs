//! Web service module

use crate::db::Database;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde_json::json;
use std::net::SocketAddr;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

/// Start the web service
pub async fn serve(listen_addr: SocketAddr, db: Database) -> anyhow::Result<()> {
    let state = AppState { db };

    let app = Router::new()
        .route("/txid/{txid}", get(get_transaction))
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
    // Try to decode the txid
    let txid_bytes = decode_txid(&txid_str)?;

    // Check the length to determine the chain
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

    tracing::debug!("Querying {} transaction: {}", chain_type, txid_str);

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
                "bundle_slot": tx.bundle_slot,
                "receipt": tx.receipt,
            });
            Ok(Json(response))
        }
        None => Err(AppError::NotFound("Transaction not found".to_string())),
    }
}

/// Decode txid from hex or base58 string
fn decode_txid(txid_str: &str) -> Result<Vec<u8>, AppError> {
    // Try hex first
    if let Ok(bytes) = hex::decode(txid_str) {
        return Ok(bytes);
    }

    // Try base58
    if let Ok(bytes) = bs58::decode(txid_str).into_vec() {
        return Ok(bytes);
    }

    Err(AppError::BadRequest(
        "Invalid txid format: must be hex or base58 encoded".to_string(),
    ))
}

/// Application error type
#[derive(Debug)]
enum AppError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

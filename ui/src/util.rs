//! Utility functions for the UI

use crate::AppError;
use bs58;
use hex;

/// Encode txid to appropriate string format based on length
/// Zcash (32 bytes) -> reverse then hex encode (standard display format)
/// Solana (64 bytes) -> base58
pub fn encode_txid(txid: &[u8]) -> String {
    match txid.len() {
        32 => {
            // Zcash: DB stores original order, display needs reversed
            let mut bytes = txid.to_vec();
            bytes.reverse();
            hex::encode(bytes)
        }
        64 => bs58::encode(txid).into_string(), // Solana
        _ => hex::encode(txid),                 // Fallback to hex
    }
}

/// Encode recipient address
pub fn encode_recipient(recipient: &[u8]) -> String {
    if recipient.len() == 32 {
        // Likely a Solana address
        bs58::encode(recipient).into_string()
    } else {
        // Try as UTF-8 string (for Zcash unified addresses)
        String::from_utf8(recipient.to_vec()).unwrap_or_else(|_| hex::encode(recipient))
    }
}

/// Decode txid from hex or base58 string
/// Zcash: hex decode then reverse (display format → original order for DB query)
/// Solana: base58 decode directly
pub fn decode_txid(txid_str: &str) -> Result<Vec<u8>, AppError> {
    // Try hex first (Zcash txids)
    if let Ok(mut bytes) = hex::decode(txid_str) {
        // Zcash: reverse from display format to original order
        bytes.reverse();
        return Ok(bytes);
    }

    // Try base58 (Solana signatures)
    if let Ok(bytes) = bs58::decode(txid_str).into_vec() {
        return Ok(bytes);
    }

    Err(AppError::BadRequest(
        "Invalid txid format: must be hex or base58 encoded".to_string(),
    ))
}

/// Parse chain from debug string
pub fn parse_chain(s: &str) -> zcore::registry::Chain {
    match s {
        "Zcash" => zcore::registry::Chain::Zcash,
        "Solana" => zcore::registry::Chain::Solana,
        _ => zcore::registry::Chain::Zcash, // Default fallback
    }
}

/// Parse coin from debug string
pub fn parse_coin(s: &str) -> zcore::registry::Coin {
    match s.to_uppercase().as_str() {
        "ZEC" => zcore::registry::Coin::Zec,
        _ => zcore::registry::Coin::Zec, // Default fallback
    }
}

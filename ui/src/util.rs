//! Utility functions for the UI

use crate::AppError;
use bs58;
use hex;

/// Encode txid to appropriate string format based on length
/// Zcash (32 bytes) -> hex, Solana (64 bytes) -> base58
pub fn encode_txid(txid: &[u8]) -> String {
    match txid.len() {
        32 => hex::encode(txid),                // Zcash
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
pub fn decode_txid(txid_str: &str) -> Result<Vec<u8>, AppError> {
    // Try hex first
    if let Ok(mut bytes) = hex::decode(txid_str) {
        bytes.reverse();
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

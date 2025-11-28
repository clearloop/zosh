//! Ed25519 signature helpers for threshold actions
//!
//! Anchor cannot generate these helpers automatically, so we provide them here.

use crate::types::MintEntry;
use anchor_client::solana_sdk::pubkey::Pubkey;

/// Create a message for signing mint action
///
/// The message format is: nonce (8 bytes) || recipient1 (32 bytes) || amount1 (8 bytes) || ...
pub fn create_mint_message(nonce: u64, mint_entries: &[MintEntry]) -> Vec<u8> {
    let mut message = nonce.to_le_bytes().to_vec();
    for entry in mint_entries {
        message.extend_from_slice(entry.recipient.as_ref());
        message.extend_from_slice(&entry.amount.to_le_bytes());
    }
    message
}

/// Create a message for signing validator update action
///
/// The message format is: nonce (8 bytes) || threshold (1 byte) || validator1 (32 bytes) || ...
pub fn create_validators_message(nonce: u64, validators: &[Pubkey], threshold: u8) -> Vec<u8> {
    let mut message = nonce.to_le_bytes().to_vec();
    message.extend_from_slice(&threshold.to_le_bytes());
    for validator in validators {
        message.extend_from_slice(validator.as_ref());
    }
    message
}

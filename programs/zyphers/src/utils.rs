use crate::errors::BridgeError;
use anchor_lang::prelude::*;

/// Action types for signature verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum ActionType {
    UpdateValidatorsFull,
    AddValidator,
    RemoveValidator,
    Mint,
}

impl ActionType {
    pub fn to_byte(&self) -> u8 {
        match self {
            ActionType::UpdateValidatorsFull => 0,
            ActionType::AddValidator => 1,
            ActionType::RemoveValidator => 2,
            ActionType::Mint => 3,
        }
    }
}

/// Verify threshold signatures for an action
pub fn verify_threshold_signatures(
    action_type: ActionType,
    nonce: u64,
    action_data: &[u8],
    signatures: &[[u8; 64]],
    validators: &[Pubkey],
    threshold: u16,
) -> Result<Vec<Pubkey>> {
    // Construct the message to sign
    let message = construct_message(action_type, nonce, action_data);
    let message_hash = simple_hash(&message);

    let mut valid_signers = Vec::new();

    for signature in signatures {
        // Try to verify the signature against each validator
        for validator in validators {
            if valid_signers.contains(validator) {
                continue; // Skip if we already verified this validator
            }

            // Verify ed25519 signature
            if ed25519_verify(signature, &message_hash, validator.as_ref()) {
                valid_signers.push(*validator);
                break;
            }
        }
    }

    // Check for duplicates (shouldn't happen with the logic above, but safety check)
    let mut sorted_signers = valid_signers.clone();
    sorted_signers.sort();
    sorted_signers.dedup();
    if sorted_signers.len() != valid_signers.len() {
        return err!(BridgeError::DuplicateSigner);
    }

    // Verify threshold is met
    if valid_signers.len() < threshold as usize {
        return err!(BridgeError::InsufficientSignatures);
    }

    Ok(valid_signers)
}

/// Construct message to be signed
fn construct_message(action_type: ActionType, nonce: u64, action_data: &[u8]) -> Vec<u8> {
    let mut message = Vec::new();
    message.push(action_type.to_byte());
    message.extend_from_slice(&nonce.to_le_bytes());
    message.extend_from_slice(action_data);
    message
}

/// Verify ed25519 signature
fn ed25519_verify(signature: &[u8; 64], message_hash: &[u8], pubkey: &[u8]) -> bool {
    if signature.len() != 64 || pubkey.len() != 32 || message_hash.is_empty() {
        return false;
    }

    // Basic validation passed
    // TODO: Implement proper Ed25519Program verification for production
    // See: https://docs.solana.com/developing/runtime-facilities/programs#ed25519-program

    // For now, we assume signatures are valid if they have correct format
    // This should be replaced with actual cryptographic verification
    true
}

/// Simple hash function for action replay protection
/// Uses a basic hash by combining the bytes - in production, use proper cryptographic hash
fn simple_hash(data: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    for (i, &byte) in data.iter().enumerate() {
        hash[i % 32] ^= byte;
    }
    // Simple mixing to spread bits
    for i in 0..32 {
        let next = (i + 1) % 32;
        hash[i] = hash[i].wrapping_add(hash[next]);
    }
    hash
}

/// Calculate action hash for replay protection
#[allow(unused)]
pub fn calculate_action_hash(action_type: ActionType, nonce: u64, action_data: &[u8]) -> [u8; 32] {
    let message = construct_message(action_type, nonce, action_data);
    simple_hash(&message)
}

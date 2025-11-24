//! Utility functions for zyphers consensus program

use crate::errors::BridgeError;
use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::{load_instruction_at_checked, ID as INSTRUCTIONS_ID},
};
use solana_sdk_ids::ed25519_program::ID as ED25519_PROGRAM_ID;

/// Action types for signature verification
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionType {
    UpdateValidatorsFull,
    AddValidator,
    RemoveValidator,
    Mint,
}

impl ActionType {
    pub fn to_byte(self) -> u8 {
        match self {
            ActionType::UpdateValidatorsFull => 0,
            ActionType::AddValidator => 1,
            ActionType::RemoveValidator => 2,
            ActionType::Mint => 3,
        }
    }
}

/// Verifies threshold signatures using Solana's Ed25519 program.
///
/// Expects Ed25519 verification instructions before this instruction in the transaction.
pub fn verify_threshold_signatures(
    action_type: ActionType,
    nonce: u64,
    action_data: &[u8],
    signatures: &[[u8; 64]],
    validators: &[Pubkey],
    threshold: u8,
    instructions_sysvar: &AccountInfo,
) -> Result<Vec<Pubkey>> {
    // Verify we have the instructions sysvar
    require_keys_eq!(
        *instructions_sysvar.key,
        INSTRUCTIONS_ID,
        BridgeError::InvalidSignature
    );

    // Construct the message that should have been signed
    let message = construct_message(action_type, nonce, action_data);

    let mut valid_signers = Vec::new();
    let current_index =
        anchor_lang::solana_program::sysvar::instructions::load_current_index_checked(
            instructions_sysvar,
        )?;

    // Check each signature by looking for corresponding Ed25519 instructions
    for (sig_index, signature) in signatures.iter().enumerate() {
        // Ed25519 instructions should come before the current instruction
        // We expect them at indices: current_index - signatures.len() + sig_index
        let ed25519_ix_index = (current_index as usize)
            .checked_sub(signatures.len())
            .and_then(|base| base.checked_add(sig_index))
            .ok_or(BridgeError::InvalidSignature)?;

        // Load the Ed25519 instruction
        let ed25519_ix = load_instruction_at_checked(ed25519_ix_index, instructions_sysvar)
            .map_err(|_| BridgeError::InvalidSignature)?;

        // Verify it's an Ed25519 instruction
        require_keys_eq!(
            ed25519_ix.program_id,
            ED25519_PROGRAM_ID,
            BridgeError::InvalidSignature
        );

        // Parse and verify the Ed25519 instruction data
        let (num_signatures, parsed_pubkey, parsed_signature, parsed_message) =
            parse_ed25519_instruction(&ed25519_ix.data)?;

        // Verify the instruction contains exactly one signature
        require_eq!(num_signatures, 1, BridgeError::InvalidSignature);

        // Verify the message matches
        require!(
            parsed_message.as_slice() == message.as_slice(),
            BridgeError::InvalidSignature
        );

        // Verify the signature matches
        require!(
            parsed_signature == *signature,
            BridgeError::InvalidSignature
        );

        // Find which validator this signature belongs to
        let validator = validators
            .iter()
            .find(|v| v.to_bytes() == parsed_pubkey)
            .ok_or(BridgeError::SignerNotValidator)?;

        // Check for duplicates
        require!(
            !valid_signers.contains(validator),
            BridgeError::DuplicateSigner
        );

        valid_signers.push(*validator);
    }

    // Verify threshold is met
    require!(
        valid_signers.len() >= threshold as usize,
        BridgeError::InsufficientSignatures
    );

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

/// Parses Ed25519 instruction data.
///
/// Returns: (num_signatures, pubkey, signature, message)
#[allow(clippy::type_complexity)]
fn parse_ed25519_instruction(data: &[u8]) -> Result<(u8, [u8; 32], [u8; 64], Vec<u8>)> {
    require!(data.len() >= 113, BridgeError::InvalidSignature);
    let num_signatures = data[0];

    // For single signature (our case), offsets are at fixed positions
    // Signature offset: bytes 1-2 (u16)
    // Public key offset: bytes 5-6 (u16)
    // Message offset: bytes 9-10 (u16)
    // Message size: bytes 11-12 (u16)
    let sig_offset = u16::from_le_bytes([data[1], data[2]]) as usize;
    let pubkey_offset = u16::from_le_bytes([data[5], data[6]]) as usize;
    let msg_offset = u16::from_le_bytes([data[9], data[10]]) as usize;
    let msg_size = u16::from_le_bytes([data[11], data[12]]) as usize;

    // Extract public key (32 bytes)
    require!(
        data.len() >= pubkey_offset + 32,
        BridgeError::InvalidSignature
    );
    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&data[pubkey_offset..pubkey_offset + 32]);

    // Extract signature (64 bytes)
    require!(data.len() >= sig_offset + 64, BridgeError::InvalidSignature);
    let mut signature = [0u8; 64];
    signature.copy_from_slice(&data[sig_offset..sig_offset + 64]);

    // Extract message
    require!(
        data.len() >= msg_offset + msg_size,
        BridgeError::InvalidSignature
    );
    let message = data[msg_offset..msg_offset + msg_size].to_vec();
    Ok((num_signatures, pubkey, signature, message))
}

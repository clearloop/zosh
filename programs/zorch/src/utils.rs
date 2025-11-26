//! Utility functions for zorch consensus program

use crate::errors::BridgeError;
use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::{load_instruction_at_checked, ID as INSTRUCTIONS_ID},
};
use solana_sdk_ids::ed25519_program::ID as ED25519_PROGRAM_ID;

const DATA_START: usize = 16;

/// Verifies threshold signatures using Solana's Ed25519 program.
///
/// Expects Ed25519 verification instructions before this instruction in the transaction.
pub fn verify_threshold_signatures(
    message: &[u8],
    signatures: &[[u8; 64]],
    validators: &[Pubkey],
    threshold: u8,
    instructions_sysvar: &AccountInfo,
) -> Result<Vec<Pubkey>> {
    require_keys_eq!(
        *instructions_sysvar.key,
        INSTRUCTIONS_ID,
        BridgeError::InvalidSignature
    );

    let mut valid_signers = Vec::new();
    let current_index =
        anchor_lang::solana_program::sysvar::instructions::load_current_index_checked(
            instructions_sysvar,
        )?;

    // Check each signature by looking for corresponding Ed25519 instructions
    for (sig_index, signature) in signatures.iter().enumerate() {
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
        require!(num_signatures == 1, BridgeError::InvalidSignature);
        require!(parsed_message == message, BridgeError::InvalidSignature);
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

/// Parses Ed25519 instruction data according to solana-ed25519-program format.
///
/// Format (from solana-ed25519-program crate):
/// - Bytes 0-1: [num_signatures: u8, padding: u8]
/// - Bytes 2-15: Ed25519SignatureOffsets struct (14 bytes):
///   - signature_offset: u16 (2 bytes)
///   - signature_instruction_index: u16 (2 bytes)
///   - public_key_offset: u16 (2 bytes)
///   - public_key_instruction_index: u16 (2 bytes)
///   - message_data_offset: u16 (2 bytes)
///   - message_data_size: u16 (2 bytes)
///   - message_instruction_index: u16 (2 bytes)
/// - Bytes 16+: actual data (pubkey, signature, message)
///
/// Constants from solana-ed25519-program:
/// - SIGNATURE_OFFSETS_START = 2
/// - SIGNATURE_OFFSETS_SERIALIZED_SIZE = 14
/// - DATA_START = 16
/// - PUBKEY_SERIALIZED_SIZE = 32
/// - SIGNATURE_SERIALIZED_SIZE = 64
///
/// Returns: (num_signatures, pubkey, signature, message)
#[allow(clippy::type_complexity)]
fn parse_ed25519_instruction(data: &[u8]) -> Result<(u8, [u8; 32], [u8; 64], Vec<u8>)> {
    require!(data.len() >= DATA_START, BridgeError::InvalidSignature);
    let num_signatures = data[0];
    require!(num_signatures == 1, BridgeError::InvalidSignature);

    // Parse Ed25519SignatureOffsets struct from bytes 2-15
    // The struct layout (14 bytes, all u16 little-endian):
    // - Bytes 2-3: signature_offset
    // - Bytes 4-5: signature_instruction_index
    // - Bytes 6-7: public_key_offset
    // - Bytes 8-9: public_key_instruction_index
    // - Bytes 10-11: message_data_offset
    // - Bytes 12-13: message_data_size
    // - Bytes 14-15: message_instruction_index
    let signature_offset = u16::from_le_bytes([data[2], data[3]]) as usize;
    let signature_instruction_index = u16::from_le_bytes([data[4], data[5]]);
    let public_key_offset = u16::from_le_bytes([data[6], data[7]]) as usize;
    let public_key_instruction_index = u16::from_le_bytes([data[8], data[9]]);
    let message_data_offset = u16::from_le_bytes([data[10], data[11]]) as usize;
    let message_data_size = u16::from_le_bytes([data[12], data[13]]) as usize;
    let message_instruction_index = u16::from_le_bytes([data[14], data[15]]);

    // Verify instruction indices are u16::MAX (meaning data is in this instruction)
    require!(
        signature_instruction_index == u16::MAX
            && public_key_instruction_index == u16::MAX
            && message_instruction_index == u16::MAX,
        BridgeError::InvalidSignature
    );

    // Use the parsed offsets
    let sig_offset = signature_offset;
    let pubkey_offset = public_key_offset;
    let msg_offset = message_data_offset;
    let msg_size = message_data_size;

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

    // Extract message using parsed offset and size
    require!(
        data.len() >= msg_offset + msg_size,
        BridgeError::InvalidSignature
    );
    let message = data[msg_offset..msg_offset + msg_size].to_vec();
    Ok((num_signatures, pubkey, signature, message))
}

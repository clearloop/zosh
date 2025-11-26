//! Ed25519 signature helpers for threshold actions
//!
//! Anchor cannot generate these helpers automatically, so we provide them here.

use super::ZorchClient;
use crate::types::MintEntry;
use anchor_client::solana_sdk::{
    ed25519_instruction::new_ed25519_instruction_with_signature, instruction::Instruction,
    pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anyhow::Result;

impl ZorchClient {
    /// Create a message for signing mint action
    ///
    /// The message format is: nonce (8 bytes) || recipient1 (32 bytes) || amount1 (8 bytes) || ...
    pub fn create_mint_message(&self, nonce: u64, mint_entries: &[MintEntry]) -> Vec<u8> {
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
    pub fn create_validators_message(
        &self,
        nonce: u64,
        new_validators: &[Pubkey],
        new_threshold: u8,
    ) -> Vec<u8> {
        let mut message = nonce.to_le_bytes().to_vec();
        message.extend_from_slice(&new_threshold.to_le_bytes());
        for validator in new_validators {
            message.extend_from_slice(validator.as_ref());
        }
        message
    }

    /// Sign a message with a keypair
    ///
    /// Returns a 64-byte signature
    pub fn sign_message(&self, message: &[u8], keypair: &Keypair) -> [u8; 64] {
        let signature = keypair.sign_message(message);
        let sig_bytes: &[u8] = signature.as_ref();
        let mut result = [0u8; 64];
        result.copy_from_slice(sig_bytes);
        result
    }

    /// Build an Ed25519 verification instruction
    ///
    /// This instruction must be included in the transaction before the program instruction
    /// that verifies the signature.
    pub fn build_ed25519_instruction(
        &self,
        message: &[u8],
        signature: &[u8; 64],
        pubkey: &Pubkey,
    ) -> Result<Instruction> {
        let pubkey_bytes = pubkey.to_bytes();
        Ok(new_ed25519_instruction_with_signature(
            message,
            signature,
            &pubkey_bytes,
        ))
    }

    /// Sign a message with multiple keypairs and build verification instructions
    ///
    /// Returns a vector of signatures and a vector of Ed25519 verification instructions.
    /// The Ed25519 instructions should be added to the transaction before the program instruction.
    pub fn sign_and_build_instructions(
        &self,
        message: &[u8],
        keypairs: &[&Keypair],
    ) -> Result<(Vec<[u8; 64]>, Vec<Instruction>)> {
        let mut signatures = Vec::new();
        let mut instructions = Vec::new();

        for keypair in keypairs {
            let sig = self.sign_message(message, keypair);
            let pubkey = keypair.pubkey();
            let instruction = self.build_ed25519_instruction(message, &sig, &pubkey)?;

            signatures.push(sig);
            instructions.push(instruction);
        }

        Ok((signatures, instructions))
    }

    /// Complete workflow: Create message, sign with validators, and return signatures
    ///
    /// This is a convenience method that combines message creation and signing for mint operations.
    pub fn prepare_mint_signatures(
        &self,
        nonce: u64,
        mint_entries: &[MintEntry],
        validator_keypairs: &[&Keypair],
    ) -> Result<Vec<[u8; 64]>> {
        let message = self.create_mint_message(nonce, mint_entries);
        let (signatures, _instructions) =
            self.sign_and_build_instructions(&message, validator_keypairs)?;
        Ok(signatures)
    }

    /// Complete workflow: Create message, sign with validators, and return signatures
    ///
    /// This is a convenience method that combines message creation and signing for validator updates.
    pub fn prepare_validators_signatures(
        &self,
        nonce: u64,
        new_validators: &[Pubkey],
        new_threshold: u8,
        validator_keypairs: &[&Keypair],
    ) -> Result<Vec<[u8; 64]>> {
        let message = self.create_validators_message(nonce, new_validators, new_threshold);
        let (signatures, _instructions) =
            self.sign_and_build_instructions(&message, validator_keypairs)?;
        Ok(signatures)
    }
}

//! client library for the zorch program
#![cfg(not(target_os = "solana"))]

use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair},
    Client, Cluster, Program,
};
use anyhow::Result;
pub use instruction::*;
use std::rc::Rc;

mod config;
pub mod instruction;
pub mod pda;
mod signatures;

/// Main client for interacting with the Zorch program
pub struct ZorchClient {
    /// Anchor client program instance
    program: Program<Rc<Keypair>>,
}

impl ZorchClient {
    /// Create a new ZorchClient
    pub fn new(cluster_url: String, ws_url: String, payer: Keypair) -> Result<Self> {
        let client = Client::new_with_options(
            Cluster::Custom(cluster_url, ws_url),
            Rc::new(payer),
            CommitmentConfig::confirmed(),
        );

        let program = client.program(crate::ID)?;

        Ok(Self { program })
    }

    /// Create a new ZorchClient with custom commitment level
    pub fn new_with_commitment(
        cluster_url: String,
        ws_url: String,
        payer: Keypair,
        commitment: CommitmentConfig,
    ) -> Result<Self> {
        let client = Client::new_with_options(
            Cluster::Custom(cluster_url, ws_url),
            Rc::new(payer),
            commitment,
        );

        let program = client.program(crate::ID)?;

        Ok(Self { program })
    }

    /// Initialize the bridge with initial validator set
    pub fn initialize(&self, validators: Vec<Pubkey>, threshold: u8) -> Result<Pubkey> {
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();
        let _tx = self
            .program
            .request()
            .accounts(crate::accounts::Initialize {
                payer: self.program.payer(),
                bridge_state,
                zec_mint,
                system_program: anchor_client::solana_sdk::system_program::ID,
                token_program: pda::TOKEN_PROGRAM,
                rent: anchor_client::solana_sdk::sysvar::rent::ID,
            })
            .args(crate::instruction::Initialize {
                validators,
                threshold,
            })
            .send()?;

        Ok(bridge_state)
    }

    /// Update token metadata (authority only)
    ///
    /// # Arguments
    /// * `name` - Token name
    /// * `symbol` - Token symbol
    /// * `uri` - Metadata URI
    pub fn update_metadata(&self, name: String, symbol: String, uri: String) -> Result<()> {
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();
        let metadata = pda::metadata();

        let _tx = self
            .program
            .request()
            .accounts(crate::accounts::UpdateMetadata {
                authority: self.program.payer(),
                bridge_state,
                zec_mint,
                metadata,
                token_metadata_program: mpl_token_metadata::ID,
                system_program: anchor_client::solana_sdk::system_program::ID,
                rent: anchor_client::solana_sdk::sysvar::rent::ID,
            })
            .args(crate::instruction::Metadata { name, symbol, uri })
            .send()?;

        Ok(())
    }

    /// Mint sZEC to recipients (threshold action)
    ///
    /// # Arguments
    /// * `mint_entries` - Recipients and amounts to mint
    /// * `validator_keypairs` - Validator keypairs for signing (must meet threshold)
    ///
    /// Note: This requires building Ed25519 verification instructions manually.
    /// Use the signature helpers in this client to create the required signatures.
    pub fn send_mint(
        &self,
        mint_entries: Vec<crate::types::MintEntry>,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        anyhow::ensure!(!mint_entries.is_empty(), "No mint entries provided");
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();

        // Get recipient token accounts
        let mut remaining_accounts = Vec::new();
        for entry in &mint_entries {
            let token_account = spl_associated_token_account::get_associated_token_address(
                &entry.recipient,
                &zec_mint,
            );
            remaining_accounts.push(anchor_client::solana_sdk::instruction::AccountMeta::new(
                token_account,
                false,
            ));
        }

        let _tx = self
            .program
            .request()
            .accounts(crate::accounts::MintZec {
                payer: self.program.payer(),
                bridge_state,
                zec_mint,
                token_program: pda::TOKEN_PROGRAM,
                system_program: anchor_client::solana_sdk::system_program::ID,
                instructions: anchor_client::solana_sdk::sysvar::instructions::ID,
            })
            .args(crate::instruction::Mint {
                mints: mint_entries,
                signatures,
            })
            .accounts(remaining_accounts)
            .send()?;

        Ok(())
    }

    /// Burn sZEC to bridge back to Zcash (public action)
    ///
    /// # Arguments
    /// * `amount` - Amount to burn
    /// * `zec_recipient` - Zcash address to receive ZEC
    pub fn send_burn(&self, amount: u64, zec_recipient: String) -> Result<()> {
        anyhow::ensure!(amount > 0, "Amount must be greater than 0");

        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();
        let signer_token_account = spl_associated_token_account::get_associated_token_address(
            &self.program.payer(),
            &zec_mint,
        );

        let _tx = self
            .program
            .request()
            .accounts(crate::accounts::BurnZec {
                signer: self.program.payer(),
                signer_token_account,
                zec_mint,
                bridge_state,
                token_program: pda::TOKEN_PROGRAM,
            })
            .args(crate::instruction::Burn {
                amount,
                zec_recipient,
            })
            .send()?;

        Ok(())
    }

    /// Update the validator set (threshold action)
    ///
    /// # Arguments
    /// * `new_validators` - New validator set
    /// * `new_threshold` - New threshold requirement
    /// * `signatures` - Signatures from current validators (must meet current threshold)
    ///
    /// Note: This requires building Ed25519 verification instructions manually.
    /// Use the signature helpers in this client to create the required signatures.
    pub fn update_validators(
        &self,
        new_validators: Vec<Pubkey>,
        new_threshold: u8,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        anyhow::ensure!(!new_validators.is_empty(), "No validators provided");
        anyhow::ensure!(
            new_threshold > 0 && new_threshold as usize <= new_validators.len(),
            "Invalid threshold"
        );

        let bridge_state = pda::bridge_state();

        let _tx = self
            .program
            .request()
            .accounts(crate::accounts::Validators {
                payer: self.program.payer(),
                bridge_state,
                system_program: anchor_client::solana_sdk::system_program::ID,
                instructions: anchor_client::solana_sdk::sysvar::instructions::ID,
            })
            .args(crate::instruction::Validators {
                new_validators,
                new_threshold,
                signatures,
            })
            .send()?;

        Ok(())
    }
}
